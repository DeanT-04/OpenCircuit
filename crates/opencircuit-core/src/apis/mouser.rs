//! Mouser API client for alternative component sourcing
//! 
//! Mouser Electronics provides an API for component search, pricing,
//! and availability information from their extensive inventory.

use super::{ApiError, BaseApiClient};
use crate::{Component, ComponentCategory, SpecValue, PriceInfo, PriceBreak, AvailabilityInfo};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Mouser API client
pub struct MouserClient {
    base_client: BaseApiClient,
    api_key: String,
}

impl MouserClient {
    pub fn new(api_key: String, rate_limit: u32, cache_ttl: u64) -> Self {
        let base_client = BaseApiClient::new(
            "mouser".to_string(),
            "https://api.mouser.com/api/v1".to_string(),
            rate_limit,
            1000, // cache capacity
            Duration::from_secs(cache_ttl),
        );

        Self {
            base_client,
            api_key,
        }
    }

    /// Search for components by keyword
    pub async fn search_components(&self, query: &str) -> Result<Vec<Component>, ApiError> {
        let search_request = MouserSearchRequest {
            search_by_keyword_request: MouserKeywordRequest {
                keyword: query.to_string(),
                records: 50,
                start_record: 0,
            },
        };

        let endpoint = format!("search/keyword?apiKey={}", self.api_key);
        let response = self.post_request(&endpoint, &search_request).await?;

        let search_response: MouserSearchResponse = serde_json::from_str(&response)
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse Mouser response: {}", e)))?;

        let mut components = Vec::new();
        if let Some(parts) = search_response.search_results.parts {
            for part in parts {
                if let Ok(component) = self.convert_mouser_part_to_component(part) {
                    components.push(component);
                }
            }
        }

        Ok(components)
    }

    /// Get detailed component information by part number
    pub async fn get_component_details(&self, part_number: &str) -> Result<Component, ApiError> {
        let search_request = MouserPartSearchRequest {
            search_by_part_request: MouserPartRequest {
                mouse_part_number: part_number.to_string(),
            },
        };

        let endpoint = format!("search/partnumber?apiKey={}", self.api_key);
        let response = self.post_request(&endpoint, &search_request).await?;

        let search_response: MouserPartSearchResponse = serde_json::from_str(&response)
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse Mouser response: {}", e)))?;

        if let Some(parts) = search_response.search_results.parts {
            if let Some(part) = parts.into_iter().next() {
                return self.convert_mouser_part_to_component(part);
            }
        }

        Err(ApiError::InvalidResponse("Component not found".to_string()))
    }

    /// Make a POST request to Mouser API
    async fn post_request<T: Serialize>(&self, endpoint: &str, body: &T) -> Result<String, ApiError> {
        self.base_client.wait_for_rate_limit().await?;

        let url = format!("{}/{}", self.base_client.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        
        let response = self.base_client.client
            .post(&url)
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

    /// Convert Mouser part to our Component model
    fn convert_mouser_part_to_component(&self, part: MouserPart) -> Result<Component, ApiError> {
        let category = self.map_mouser_category(&part.category);
        
        let mut component = Component::new(
            part.manufacturer_part_number,
            part.manufacturer,
            category,
            part.description,
        );

        // Add specifications from product attributes
        let mut specifications = HashMap::new();
        for attr in part.product_attributes {
            let value = SpecValue::String(attr.attribute_value);
            specifications.insert(attr.attribute_name, value);
        }
        component.specifications = specifications;

        // Add datasheet URL
        if let Some(datasheet) = part.data_sheet_url {
            if !datasheet.is_empty() {
                component.datasheet_url = Some(datasheet);
            }
        }

        // Add pricing information
        if !part.price_breaks.is_empty() {
            let price_breaks: Vec<PriceBreak> = part.price_breaks.clone()
                .into_iter()
                .map(|p| PriceBreak {
                    quantity: p.quantity,
                    unit_price: p.price.parse().unwrap_or(0.0),
                })
                .collect();

            component.price_info = Some(PriceInfo {
                currency: part.price_breaks.first()
                    .and_then(|p| p.currency.clone())
                    .unwrap_or_else(|| "USD".to_string()),
                price_breaks,
                last_updated: Utc::now(),
                supplier: "Mouser".to_string(),
            });
        }

        // Add availability information
        let availability_text = part.availability.unwrap_or_else(|| "Unknown".to_string());
        let in_stock = availability_text.to_lowercase().contains("in stock") || 
                      availability_text.parse::<u32>().unwrap_or(0) > 0;

        component.availability = Some(AvailabilityInfo {
            in_stock,
            quantity_available: availability_text.parse().ok(),
            lead_time_days: part.lead_time.and_then(|lt| lt.parse().ok()),
            minimum_order_quantity: part.min_order_quantity.and_then(|moq| moq.parse().ok()),
            last_updated: Utc::now(),
            supplier: "Mouser".to_string(),
        });

        Ok(component)
    }

    /// Map Mouser category to our ComponentCategory enum
    fn map_mouser_category(&self, category: &str) -> ComponentCategory {
        match category.to_lowercase().as_str() {
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
            _ => ComponentCategory::Custom(category.to_string()),
        }
    }
}

// Mouser API structures

#[derive(Debug, Serialize)]
struct MouserSearchRequest {
    #[serde(rename = "SearchByKeywordRequest")]
    search_by_keyword_request: MouserKeywordRequest,
}

#[derive(Debug, Serialize)]
struct MouserKeywordRequest {
    keyword: String,
    records: u32,
    #[serde(rename = "startRecord")]
    start_record: u32,
}

#[derive(Debug, Serialize)]
struct MouserPartSearchRequest {
    #[serde(rename = "SearchByPartRequest")]
    search_by_part_request: MouserPartRequest,
}

#[derive(Debug, Serialize)]
struct MouserPartRequest {
    #[serde(rename = "mouserPartNumber")]
    mouse_part_number: String,
}

#[derive(Debug, Deserialize)]
struct MouserSearchResponse {
    #[serde(rename = "SearchResults")]
    search_results: MouserSearchResults,
}

#[derive(Debug, Deserialize)]
struct MouserPartSearchResponse {
    #[serde(rename = "SearchResults")]
    search_results: MouserPartSearchResults,
}

#[derive(Debug, Deserialize)]
struct MouserSearchResults {
    #[serde(rename = "Parts")]
    parts: Option<Vec<MouserPart>>,
}

#[derive(Debug, Deserialize)]
struct MouserPartSearchResults {
    #[serde(rename = "Parts")]
    parts: Option<Vec<MouserPart>>,
}

#[derive(Debug, Deserialize)]
struct MouserPart {
    #[serde(rename = "ManufacturerPartNumber")]
    manufacturer_part_number: String,
    #[serde(rename = "Manufacturer")]
    manufacturer: String,
    #[serde(rename = "Category")]
    category: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "ProductAttributes")]
    product_attributes: Vec<MouserAttribute>,
    #[serde(rename = "DataSheetUrl")]
    data_sheet_url: Option<String>,
    #[serde(rename = "PriceBreaks")]
    price_breaks: Vec<MouserPriceBreak>,
    #[serde(rename = "Availability")]
    availability: Option<String>,
    #[serde(rename = "LeadTime")]
    lead_time: Option<String>,
    #[serde(rename = "MinOrderQty")]
    min_order_quantity: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MouserAttribute {
    #[serde(rename = "AttributeName")]
    attribute_name: String,
    #[serde(rename = "AttributeValue")]
    attribute_value: String,
}

#[derive(Debug, Deserialize, Clone)]
struct MouserPriceBreak {
    #[serde(rename = "Quantity")]
    quantity: u32,
    #[serde(rename = "Price")]
    price: String,
    #[serde(rename = "Currency")]
    currency: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_mapping() {
        let client = MouserClient::new("test_key".to_string(), 100, 3600);
        
        assert_eq!(client.map_mouser_category("Resistors"), ComponentCategory::Resistors);
        assert_eq!(client.map_mouser_category("Custom Component"), ComponentCategory::Custom("Custom Component".to_string()));
    }

    #[test]
    fn test_client_creation() {
        let client = MouserClient::new("test_key".to_string(), 100, 3600);
        assert_eq!(client.api_key, "test_key");
        assert_eq!(client.base_client.service_name, "mouser");
    }

    #[tokio::test]
    async fn test_search_request_serialization() {
        let request = MouserSearchRequest {
            search_by_keyword_request: MouserKeywordRequest {
                keyword: "resistor".to_string(),
                records: 50,
                start_record: 0,
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("SearchByKeywordRequest"));
        assert!(json.contains("resistor"));
    }
}