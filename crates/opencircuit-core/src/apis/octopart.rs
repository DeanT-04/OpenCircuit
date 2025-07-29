//! Octopart API client for component search and aggregation
//! 
//! Octopart is a search engine for electronic components that aggregates data
//! from multiple suppliers and provides comprehensive component information.

use super::{ApiError, BaseApiClient};
use crate::{Component, ComponentCategory, SpecValue, PriceInfo, PriceBreak, AvailabilityInfo};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Octopart API client
pub struct OctopartClient {
    base_client: BaseApiClient,
    api_key: String,
}

impl OctopartClient {
    pub fn new(api_key: String, rate_limit: u32, cache_ttl: u64) -> Self {
        let base_client = BaseApiClient::new(
            "octopart".to_string(),
            "https://octopart.com/api/v4".to_string(),
            rate_limit,
            1000, // cache capacity
            Duration::from_secs(cache_ttl),
        );

        Self {
            base_client,
            api_key,
        }
    }

    /// Search for components by query string
    pub async fn search_components(&self, query: &str) -> Result<Vec<Component>, ApiError> {
        let endpoint = format!(
            "search?q={}&apikey={}&include[]=specs&include[]=datasheets&include[]=offers",
            urlencoding::encode(query),
            self.api_key
        );

        let cache_key = format!("octopart_search_{}", query);
        let response_text = self.base_client.cached_get(&endpoint, &cache_key).await?;

        let search_response: OctopartSearchResponse = serde_json::from_str(&response_text)
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse Octopart response: {}", e)))?;

        let mut components = Vec::new();
        for result in search_response.results {
            if let Ok(component) = self.convert_octopart_part_to_component(result.item) {
                components.push(component);
            }
        }

        Ok(components)
    }

    /// Get detailed component information by part number
    pub async fn get_component_details(&self, part_number: &str) -> Result<Component, ApiError> {
        let endpoint = format!(
            "search?q={}&apikey={}&include[]=specs&include[]=datasheets&include[]=offers&limit=1",
            urlencoding::encode(part_number),
            self.api_key
        );

        let cache_key = format!("octopart_details_{}", part_number);
        let response_text = self.base_client.cached_get(&endpoint, &cache_key).await?;

        let search_response: OctopartSearchResponse = serde_json::from_str(&response_text)
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse Octopart response: {}", e)))?;

        if let Some(result) = search_response.results.into_iter().next() {
            self.convert_octopart_part_to_component(result.item)
        } else {
            Err(ApiError::InvalidResponse("Component not found".to_string()))
        }
    }

    /// Convert Octopart part data to our Component model
    fn convert_octopart_part_to_component(&self, part: OctopartPart) -> Result<Component, ApiError> {
        let category = self.map_octopart_category(&part.category);
        
        let mut component = Component::new(
            part.mpn,
            part.manufacturer.name,
            category,
            part.short_description.unwrap_or_else(|| "No description available".to_string()),
        );

        // Add specifications
        let mut specifications = HashMap::new();
        for spec in part.specs {
            let value = match spec.value {
                OctopartSpecValue::String(s) => SpecValue::String(s),
                OctopartSpecValue::Number(n) => SpecValue::Number(n),
                OctopartSpecValue::Array(arr) => SpecValue::List(arr),
            };
            specifications.insert(spec.attribute.name, value);
        }
        component.specifications = specifications;

        // Add datasheet URL
        if let Some(datasheet) = part.datasheets.into_iter().next() {
            component.datasheet_url = Some(datasheet.url);
        }

        // Add pricing information from offers
        if let Some(offer) = part.offers.into_iter().next() {
            let supplier_name = offer.supplier.name.clone();
            let prices_clone = offer.prices.clone();
            let price_breaks: Vec<PriceBreak> = offer.prices
                .into_iter()
                .map(|p| PriceBreak {
                    quantity: p.quantity,
                    unit_price: p.price,
                })
                .collect();

            if !price_breaks.is_empty() {
                component.price_info = Some(PriceInfo {
                    currency: prices_clone.first().map(|p| p.currency.clone()).unwrap_or_else(|| "USD".to_string()),
                    price_breaks,
                    last_updated: Utc::now(),
                    supplier: supplier_name.clone(),
                });
            }

            // Add availability information
            component.availability = Some(AvailabilityInfo {
                in_stock: offer.in_stock_quantity.unwrap_or(0) > 0,
                quantity_available: offer.in_stock_quantity,
                lead_time_days: None, // Octopart doesn't provide lead time
                minimum_order_quantity: offer.moq,
                last_updated: Utc::now(),
                supplier: supplier_name,
            });
        }

        Ok(component)
    }

    /// Map Octopart category to our ComponentCategory enum
    fn map_octopart_category(&self, category: &OctopartCategory) -> ComponentCategory {
        match category.name.to_lowercase().as_str() {
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
            _ => ComponentCategory::Custom(category.name.clone()),
        }
    }
}

// Octopart API response structures

#[derive(Debug, Deserialize)]
struct OctopartSearchResponse {
    results: Vec<OctopartSearchResult>,
}

#[derive(Debug, Deserialize)]
struct OctopartSearchResult {
    item: OctopartPart,
}

#[derive(Debug, Deserialize)]
struct OctopartPart {
    mpn: String,
    manufacturer: OctopartManufacturer,
    category: OctopartCategory,
    short_description: Option<String>,
    specs: Vec<OctopartSpec>,
    datasheets: Vec<OctopartDatasheet>,
    offers: Vec<OctopartOffer>,
}

#[derive(Debug, Deserialize)]
struct OctopartManufacturer {
    name: String,
}

#[derive(Debug, Deserialize)]
struct OctopartCategory {
    name: String,
}

#[derive(Debug, Deserialize)]
struct OctopartSpec {
    attribute: OctopartAttribute,
    value: OctopartSpecValue,
}

#[derive(Debug, Deserialize)]
struct OctopartAttribute {
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum OctopartSpecValue {
    String(String),
    Number(f64),
    Array(Vec<String>),
}

#[derive(Debug, Deserialize)]
struct OctopartDatasheet {
    url: String,
}

#[derive(Debug, Deserialize)]
struct OctopartOffer {
    supplier: OctopartSupplier,
    prices: Vec<OctopartPrice>,
    in_stock_quantity: Option<u32>,
    moq: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct OctopartSupplier {
    name: String,
}

#[derive(Debug, Deserialize, Clone)]
struct OctopartPrice {
    quantity: u32,
    price: f64,
    currency: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_mapping() {
        let client = OctopartClient::new("test_key".to_string(), 100, 3600);
        
        let resistor_category = OctopartCategory {
            name: "Resistors".to_string(),
        };
        assert_eq!(client.map_octopart_category(&resistor_category), ComponentCategory::Resistors);
        
        let custom_category = OctopartCategory {
            name: "Custom Component".to_string(),
        };
        assert_eq!(client.map_octopart_category(&custom_category), ComponentCategory::Custom("Custom Component".to_string()));
    }

    #[tokio::test]
    async fn test_client_creation() {
        let client = OctopartClient::new("test_key".to_string(), 100, 3600);
        assert_eq!(client.api_key, "test_key");
        assert_eq!(client.base_client.service_name, "octopart");
    }
}