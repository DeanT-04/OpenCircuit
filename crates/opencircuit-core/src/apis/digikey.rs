//! DigiKey API client for direct supplier integration
//! 
//! DigiKey provides a comprehensive API for component search, pricing,
//! and availability information directly from their inventory.

use super::{ApiError, BaseApiClient};
use crate::models::{Component, ComponentCategory, SpecValue, PriceInfo, PriceBreak, AvailabilityInfo};
use anyhow::Result;
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use std::cell::RefCell;

/// DigiKey API client with OAuth 2.0 authentication
pub struct DigiKeyClient {
    base_client: BaseApiClient,
    client_id: String,
    client_secret: String,
    access_token: RefCell<Option<String>>,
    token_expires_at: RefCell<Option<DateTime<Utc>>>,
    sandbox_mode: bool,
}

impl DigiKeyClient {
    pub fn new(client_id: String, client_secret: String, sandbox: bool, rate_limit: u32, cache_ttl: u64) -> Self {
        let base_url = if sandbox {
            "https://sandbox-api.digikey.com"
        } else {
            "https://api.digikey.com"
        };

        let base_client = BaseApiClient::new(
            "digikey".to_string(),
            base_url.to_string(),
            rate_limit,
            1000, // cache capacity
            Duration::from_secs(cache_ttl),
        );

        Self {
            base_client,
            client_id,
            client_secret,
            access_token: RefCell::new(None),
            token_expires_at: RefCell::new(None),
            sandbox_mode: sandbox,
        }
    }

    /// Authenticate with DigiKey OAuth 2.0
    async fn authenticate(&self) -> Result<(), ApiError> {
        // Check if we have a valid token
        {
            let access_token = self.access_token.borrow();
            let token_expires_at = self.token_expires_at.borrow();
            if let (Some(_), Some(expires_at)) = (access_token.as_ref(), token_expires_at.as_ref()) {
                if *expires_at > Utc::now() + chrono::Duration::minutes(5) {
                    return Ok(()); // Token is still valid
                }
            }
        }

        // Request new token using client credentials flow
        let auth_header = general_purpose::STANDARD.encode(
            format!("{}:{}", self.client_id, self.client_secret)
        );

        let token_url = format!("{}/v1/oauth2/token", self.base_client.base_url);
        
        let params = [
            ("grant_type", "client_credentials"),
        ];

        let response = self.base_client.client
            .post(&token_url)
            .header("Authorization", format!("Basic {}", auth_header))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ApiError::AuthenticationFailed {
                service: "DigiKey".to_string(),
                reason: format!("HTTP {}", response.status()),
            });
        }

        let token_response: DigiKeyTokenResponse = response
            .json()
            .await
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse token response: {}", e)))?;

        *self.access_token.borrow_mut() = Some(token_response.access_token);
        *self.token_expires_at.borrow_mut() = Some(Utc::now() + chrono::Duration::seconds(token_response.expires_in as i64));

        Ok(())
    }

    /// Search for components by keyword
    pub async fn search_components(&self, query: &str) -> Result<Vec<Component>, ApiError> {
        self.authenticate().await?;

        let search_request = DigiKeySearchRequest {
            keywords: query.to_string(),
            record_count: 50,
            record_start_pos: 0,
            filters: HashMap::new(),
            sort: DigiKeySort {
                sort_option: "SortByUnitPrice".to_string(),
                direction: "Ascending".to_string(),
            },
            requested_quantity: 1,
        };

        let endpoint = "/Search/v3/Products/Keyword";
        let response = self.authenticated_post(endpoint, &search_request).await?;

        let search_response: DigiKeySearchResponse = serde_json::from_str(&response)
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse DigiKey response: {}", e)))?;

        let mut components = Vec::new();
        for product in search_response.products {
            if let Ok(component) = self.convert_digikey_product_to_component(product) {
                components.push(component);
            }
        }

        Ok(components)
    }

    /// Get detailed component information by part number
    pub async fn get_component_details(&self, part_number: &str) -> Result<Component, ApiError> {
        self.authenticate().await?;

        let endpoint = format!("/Search/v3/Products/{}", urlencoding::encode(part_number));
        let response = self.authenticated_get(&endpoint).await?;

        let product: DigiKeyProduct = serde_json::from_str(&response)
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse DigiKey product: {}", e)))?;

        self.convert_digikey_product_to_component(product)
    }

    /// Make authenticated GET request
    async fn authenticated_get(&self, endpoint: &str) -> Result<String, ApiError> {
        let token = self.access_token.borrow()
            .as_ref()
            .ok_or_else(|| ApiError::AuthenticationFailed {
                service: "DigiKey".to_string(),
                reason: "No access token available".to_string(),
            })?
            .clone();

        self.base_client.wait_for_rate_limit().await?;

        let url = format!("{}{}", self.base_client.base_url, endpoint);
        
        let response = self.base_client.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("X-DIGIKEY-Client-Id", &self.client_id)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ApiError::InvalidResponse(
                format!("HTTP {}: {}", response.status(), response.status().canonical_reason().unwrap_or("Unknown"))
            ));
        }

        response.text().await
            .map_err(|e| ApiError::NetworkError(e.to_string()))
    }

    /// Make authenticated POST request
    async fn authenticated_post<T: Serialize>(&self, endpoint: &str, body: &T) -> Result<String, ApiError> {
        let token = self.access_token.borrow()
            .as_ref()
            .ok_or_else(|| ApiError::AuthenticationFailed {
                service: "DigiKey".to_string(),
                reason: "No access token available".to_string(),
            })?
            .clone();

        self.base_client.wait_for_rate_limit().await?;

        let url = format!("{}{}", self.base_client.base_url, endpoint);
        
        let response = self.base_client.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("X-DIGIKEY-Client-Id", &self.client_id)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ApiError::InvalidResponse(
                format!("HTTP {}: {}", response.status(), response.status().canonical_reason().unwrap_or("Unknown"))
            ));
        }

        response.text().await
            .map_err(|e| ApiError::NetworkError(e.to_string()))
    }

    /// Convert DigiKey product to our Component model
    fn convert_digikey_product_to_component(&self, product: DigiKeyProduct) -> Result<Component, ApiError> {
        let category = self.map_digikey_category(&product.category);
        
        let mut component = Component::new(
            product.manufacturer_part_number,
            product.manufacturer.value,
            category,
            product.product_description,
        );

        // Add specifications from parameters
        let mut specifications = HashMap::new();
        for param in product.parameters {
            let value = SpecValue::String(param.value);
            specifications.insert(param.parameter, value);
        }
        component.specifications = specifications;

        // Add datasheet URL
        if let Some(datasheet) = product.primary_datasheet {
            component.datasheet_url = Some(datasheet);
        }

        // Add pricing information
        if !product.standard_pricing.is_empty() {
            let price_breaks: Vec<PriceBreak> = product.standard_pricing
                .into_iter()
                .map(|p| PriceBreak {
                    quantity: p.break_quantity,
                    unit_price: p.unit_price,
                })
                .collect();

            component.price_info = Some(PriceInfo {
                currency: "USD".to_string(), // DigiKey typically uses USD
                price_breaks,
                last_updated: Utc::now(),
                supplier: "DigiKey".to_string(),
            });
        }

        // Add availability information
        component.availability = Some(AvailabilityInfo {
            in_stock: product.quantity_available > 0,
            quantity_available: Some(product.quantity_available),
            lead_time_days: None, // Would need additional API call
            minimum_order_quantity: Some(product.minimum_order_quantity),
            last_updated: Utc::now(),
            supplier: "DigiKey".to_string(),
        });

        Ok(component)
    }

    /// Map DigiKey category to our ComponentCategory enum
    fn map_digikey_category(&self, category: &DigiKeyCategory) -> ComponentCategory {
        match category.value.to_lowercase().as_str() {
            name if name.contains("resistor") => ComponentCategory::Resistors,
            name if name.contains("capacitor") => ComponentCategory::Capacitors,
            name if name.contains("inductor") => ComponentCategory::Inductors,
            name if name.contains("diode") => ComponentCategory::Diodes,
            name if name.contains("transistor") => ComponentCategory::Transistors,
            name if name.contains("ic") || name.contains("integrated") => ComponentCategory::IntegratedCircuits,
            name if name.contains("connector") => ComponentCategory::Connectors,
            name if name.contains("switch") => ComponentCategory::Switches,
            name if name.contains("crystal") || name.contains("oscillator") => ComponentCategory::Crystals,
            name if name.contains("sensor") => ComponentCategory::Sensors,
            name if name.contains("power") => ComponentCategory::Power,
            _ => ComponentCategory::Custom(category.value.clone()),
        }
    }
}

// DigiKey API structures

#[derive(Debug, Deserialize)]
struct DigiKeyTokenResponse {
    access_token: String,
    expires_in: u64,
    token_type: String,
}

#[derive(Debug, Serialize)]
struct DigiKeySearchRequest {
    keywords: String,
    record_count: u32,
    record_start_pos: u32,
    filters: HashMap<String, String>,
    sort: DigiKeySort,
    requested_quantity: u32,
}

#[derive(Debug, Serialize)]
struct DigiKeySort {
    sort_option: String,
    direction: String,
}

#[derive(Debug, Deserialize)]
struct DigiKeySearchResponse {
    products: Vec<DigiKeyProduct>,
}

#[derive(Debug, Deserialize)]
struct DigiKeyProduct {
    #[serde(rename = "ManufacturerPartNumber")]
    manufacturer_part_number: String,
    #[serde(rename = "Manufacturer")]
    manufacturer: DigiKeyManufacturer,
    #[serde(rename = "Category")]
    category: DigiKeyCategory,
    #[serde(rename = "ProductDescription")]
    product_description: String,
    #[serde(rename = "Parameters")]
    parameters: Vec<DigiKeyParameter>,
    #[serde(rename = "PrimaryDatasheet")]
    primary_datasheet: Option<String>,
    #[serde(rename = "StandardPricing")]
    standard_pricing: Vec<DigiKeyPricing>,
    #[serde(rename = "QuantityAvailable")]
    quantity_available: u32,
    #[serde(rename = "MinimumOrderQuantity")]
    minimum_order_quantity: u32,
}

#[derive(Debug, Deserialize)]
struct DigiKeyManufacturer {
    #[serde(rename = "Value")]
    value: String,
}

#[derive(Debug, Deserialize)]
struct DigiKeyCategory {
    #[serde(rename = "Value")]
    value: String,
}

#[derive(Debug, Deserialize)]
struct DigiKeyParameter {
    #[serde(rename = "Parameter")]
    parameter: String,
    #[serde(rename = "Value")]
    value: String,
}

#[derive(Debug, Deserialize, Clone)]
struct DigiKeyPricing {
    #[serde(rename = "BreakQuantity")]
    break_quantity: u32,
    #[serde(rename = "UnitPrice")]
    unit_price: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_mapping() {
        let client = DigiKeyClient::new(
            "test_id".to_string(),
            "test_secret".to_string(),
            true,
            100,
            3600
        );
        
        let resistor_category = DigiKeyCategory {
            value: "Resistors".to_string(),
        };
        assert_eq!(client.map_digikey_category(&resistor_category), ComponentCategory::Resistors);
    }

    #[test]
    fn test_client_creation() {
        let client = DigiKeyClient::new(
            "test_id".to_string(),
            "test_secret".to_string(),
            true,
            100,
            3600
        );
        assert_eq!(client.client_id, "test_id");
        assert_eq!(client.client_secret, "test_secret");
        assert!(client.sandbox_mode);
    }
}