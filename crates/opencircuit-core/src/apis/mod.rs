//! External API integrations for component data
//! 
//! This module provides clients for various component supplier APIs including:
//! - Octopart: Primary component search and aggregation
//! - DigiKey: Direct supplier integration
//! - Mouser: Alternative supplier integration
//! 
//! Features:
//! - Rate limiting and quota management
//! - Intelligent caching
//! - Error handling and retry strategies
//! - OAuth and API key authentication

use anyhow::Result;
use chrono::{DateTime, Utc};
use governor::{Quota, RateLimiter};
use lru::LruCache;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::DefaultHasher;
use std::num::{NonZeroU32, NonZeroUsize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;

// Type alias for the keyed rate limiter
type KeyedRateLimiter = RateLimiter<String, governor::state::keyed::DashMapStateStore<String>, governor::clock::DefaultClock>;

pub mod octopart;
pub mod digikey;
pub mod mouser;

pub use octopart::OctopartClient;
pub use digikey::DigiKeyClient;
pub use mouser::MouserClient;

/// API-specific errors
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Rate limit exceeded for {service}")]
    RateLimitExceeded { service: String },
    
    #[error("Authentication failed for {service}: {reason}")]
    AuthenticationFailed { service: String, reason: String },
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("API quota exceeded for {service}")]
    QuotaExceeded { service: String },
    
    #[error("Invalid API response: {0}")]
    InvalidResponse(String),
    
    #[error("Service unavailable: {service}")]
    ServiceUnavailable { service: String },
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// API key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub service: String,
    pub key: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub rate_limit: Option<RateLimit>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub burst_size: Option<u32>,
}

/// Cached API response
#[derive(Debug, Clone)]
pub struct CachedResponse {
    pub data: String,
    pub expires_at: DateTime<Utc>,
    pub etag: Option<String>,
}

/// API cache manager
pub struct ApiCache {
    memory_cache: Arc<Mutex<LruCache<String, CachedResponse>>>,
    default_ttl: Duration,
}

impl ApiCache {
    pub fn new(capacity: usize, default_ttl: Duration) -> Self {
        Self {
            memory_cache: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(capacity).unwrap()
            ))),
            default_ttl,
        }
    }

    pub fn get(&self, key: &str) -> Option<CachedResponse> {
        let mut cache = self.memory_cache.lock().unwrap();
        if let Some(response) = cache.get(key) {
            if response.expires_at > Utc::now() {
                return Some(response.clone());
            } else {
                cache.pop(key);
            }
        }
        None
    }

    pub fn set(&self, key: String, data: String, ttl: Option<Duration>) {
        let ttl = ttl.unwrap_or(self.default_ttl);
        let expires_at = Utc::now() + chrono::Duration::from_std(ttl).unwrap();
        
        let response = CachedResponse {
            data,
            expires_at,
            etag: None,
        };

        let mut cache = self.memory_cache.lock().unwrap();
        cache.put(key, response);
    }

    pub fn invalidate(&self, key: &str) {
        let mut cache = self.memory_cache.lock().unwrap();
        cache.pop(key);
    }

    pub fn clear(&self) {
        let mut cache = self.memory_cache.lock().unwrap();
        cache.clear();
    }
}

/// Base API client with common functionality
pub struct BaseApiClient {
    pub client: Client,
    pub rate_limiter: Arc<KeyedRateLimiter>,
    pub cache: ApiCache,
    pub base_url: String,
    pub service_name: String,
}

impl BaseApiClient {
    pub fn new(
        service_name: String,
        base_url: String,
        requests_per_minute: u32,
        cache_capacity: usize,
        cache_ttl: Duration,
    ) -> Self {
        let quota = Quota::per_minute(NonZeroU32::new(requests_per_minute).unwrap());
        let rate_limiter = Arc::new(RateLimiter::keyed(quota));
        
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("OpenCircuit/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            rate_limiter,
            cache: ApiCache::new(cache_capacity, cache_ttl),
            base_url,
            service_name,
        }
    }

    /// Wait for rate limit if necessary
    pub async fn wait_for_rate_limit(&self) -> Result<(), ApiError> {
        match self.rate_limiter.check_key(&self.service_name) {
            Ok(_) => Ok(()),
            Err(_) => {
                tracing::warn!("Rate limit exceeded for {}, waiting...", self.service_name);
                sleep(Duration::from_secs(1)).await;
                Err(ApiError::RateLimitExceeded {
                    service: self.service_name.clone(),
                })
            }
        }
    }

    /// Make a cached GET request
    pub async fn cached_get(&self, endpoint: &str, cache_key: &str) -> Result<String, ApiError> {
        // Check cache first
        if let Some(cached) = self.cache.get(cache_key) {
            tracing::debug!("Cache hit for {}", cache_key);
            return Ok(cached.data);
        }

        // Wait for rate limit
        self.wait_for_rate_limit().await?;

        // Make request
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        
        tracing::debug!("Making API request to: {}", url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ApiError::InvalidResponse(
                format!("HTTP {}: {}", response.status(), response.status().canonical_reason().unwrap_or("Unknown"))
            ));
        }

        let data = response
            .text()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        // Cache the response
        self.cache.set(cache_key.to_string(), data.clone(), None);

        Ok(data)
    }
}

/// API configuration manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub octopart: Option<OctopartConfig>,
    pub digikey: Option<DigiKeyConfig>,
    pub mouser: Option<MouserConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OctopartConfig {
    pub enabled: bool,
    pub api_key: String,
    pub rate_limit: u32,
    pub cache_ttl: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigiKeyConfig {
    pub enabled: bool,
    pub client_id: String,
    pub client_secret: String,
    pub sandbox: bool,
    pub rate_limit: u32,
    pub cache_ttl: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouserConfig {
    pub enabled: bool,
    pub api_key: String,
    pub rate_limit: u32,
    pub cache_ttl: u64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            octopart: Some(OctopartConfig {
                enabled: false,
                api_key: String::new(),
                rate_limit: 100,
                cache_ttl: 3600,
            }),
            digikey: Some(DigiKeyConfig {
                enabled: false,
                client_id: String::new(),
                client_secret: String::new(),
                sandbox: true,
                rate_limit: 100,
                cache_ttl: 3600,
            }),
            mouser: Some(MouserConfig {
                enabled: false,
                api_key: String::new(),
                rate_limit: 100,
                cache_ttl: 3600,
            }),
        }
    }
}

/// Unified API manager for all component suppliers
pub struct ApiManager {
    pub octopart: Option<OctopartClient>,
    pub digikey: Option<DigiKeyClient>,
    pub mouser: Option<MouserClient>,
}

impl ApiManager {
    pub fn new(config: ApiConfig) -> Self {
        let octopart = config.octopart
            .filter(|c| c.enabled && !c.api_key.is_empty())
            .map(|c| OctopartClient::new(c.api_key, c.rate_limit, c.cache_ttl));

        let digikey = config.digikey
            .filter(|c| c.enabled && !c.client_id.is_empty())
            .map(|c| DigiKeyClient::new(c.client_id, c.client_secret, c.sandbox, c.rate_limit, c.cache_ttl));

        let mouser = config.mouser
            .filter(|c| c.enabled && !c.api_key.is_empty())
            .map(|c| MouserClient::new(c.api_key, c.rate_limit, c.cache_ttl));

        Self {
            octopart,
            digikey,
            mouser,
        }
    }

    /// Search components across all enabled APIs
    pub async fn search_components(&self, query: &str) -> Result<Vec<crate::Component>, ApiError> {
        let mut all_components = Vec::new();

        // Search Octopart
        if let Some(ref client) = self.octopart {
            match client.search_components(query).await {
                Ok(mut components) => all_components.append(&mut components),
                Err(e) => tracing::warn!("Octopart search failed: {}", e),
            }
        }

        // Search DigiKey
        if let Some(ref client) = self.digikey {
            match client.search_components(query).await {
                Ok(mut components) => all_components.append(&mut components),
                Err(e) => tracing::warn!("DigiKey search failed: {}", e),
            }
        }

        // Search Mouser
        if let Some(ref client) = self.mouser {
            match client.search_components(query).await {
                Ok(mut components) => all_components.append(&mut components),
                Err(e) => tracing::warn!("Mouser search failed: {}", e),
            }
        }

        // Remove duplicates based on part number and manufacturer
        all_components.sort_by(|a, b| {
            a.part_number.cmp(&b.part_number)
                .then_with(|| a.manufacturer.cmp(&b.manufacturer))
        });
        all_components.dedup_by(|a, b| {
            a.part_number == b.part_number && a.manufacturer == b.manufacturer
        });

        Ok(all_components)
    }

    /// Get component details by part number
    pub async fn get_component_details(&self, part_number: &str) -> Result<Option<crate::Component>, ApiError> {
        // Try each API in order of preference
        if let Some(ref client) = self.octopart {
            if let Ok(component) = client.get_component_details(part_number).await {
                return Ok(Some(component));
            }
        }

        if let Some(ref client) = self.digikey {
            if let Ok(component) = client.get_component_details(part_number).await {
                return Ok(Some(component));
            }
        }

        if let Some(ref client) = self.mouser {
            if let Ok(component) = client.get_component_details(part_number).await {
                return Ok(Some(component));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_cache() {
        let cache = ApiCache::new(10, Duration::from_secs(60));
        
        cache.set("test_key".to_string(), "test_data".to_string(), None);
        
        let result = cache.get("test_key");
        assert!(result.is_some());
        assert_eq!(result.unwrap().data, "test_data");
        
        cache.invalidate("test_key");
        assert!(cache.get("test_key").is_none());
    }

    #[test]
    fn test_api_config_default() {
        let config = ApiConfig::default();
        assert!(config.octopart.is_some());
        assert!(config.digikey.is_some());
        assert!(config.mouser.is_some());
    }
}