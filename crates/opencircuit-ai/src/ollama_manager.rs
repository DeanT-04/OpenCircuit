//! Ollama model manager for OpenCircuit
//! 
//! This module handles model downloading, management, and automatic
//! selection based on use case and system capabilities.

use crate::models::*;
use crate::ollama_client::{OpenCircuitOllamaClient, OllamaConfig};
use opencircuit_core::OpenCircuitError;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{info, warn, error, debug};
use chrono::Utc;

/// Result type for Ollama operations
type OllamaResult<T> = std::result::Result<T, OpenCircuitError>;

/// Ollama model manager for OpenCircuit
pub struct OllamaManager {
    /// Current model status
    status: ModelStatus,
    /// Ollama client for API interactions
    client: OpenCircuitOllamaClient,
    /// Configuration
    config: OllamaConfig,
    /// Performance tracking
    performance_tracker: HashMap<AiModel, ModelPerformance>,
}

impl OllamaManager {
    /// Create a new Ollama manager
    pub fn new() -> Self {
        let config = OllamaConfig::default();
        let client = OpenCircuitOllamaClient::with_config(config.clone());
        
        Self {
            status: ModelStatus::default(),
            client,
            config,
            performance_tracker: HashMap::new(),
        }
    }

    /// Create a new Ollama manager with custom configuration
    pub fn with_config(config: OllamaConfig) -> Self {
        let client = OpenCircuitOllamaClient::with_config(config.clone());
        
        Self {
            status: ModelStatus::default(),
            client,
            config,
            performance_tracker: HashMap::new(),
        }
    }

    /// Initialize the manager and check server status
    pub async fn initialize(&mut self) -> OllamaResult<()> {
        info!("Initializing Ollama manager...");
        
        // Check if Ollama server is running
        self.check_server_status().await?;
        
        // Scan for available models
        self.scan_available_models().await?;
        
        // Set up default model if available
        self.setup_default_model().await?;
        
        info!("Ollama manager initialized successfully");
        Ok(())
    }

    /// Check if Ollama server is running and accessible
    pub async fn check_server_status(&mut self) -> OllamaResult<ServerStatus> {
        debug!("Checking Ollama server status...");
        
        let is_healthy = self.client.health_check().await.unwrap_or(false);
        self.status.server_status = if is_healthy {
            info!("Ollama server is running and accessible");
            ServerStatus::Running
        } else {
            warn!("Ollama server is not accessible");
            ServerStatus::Stopped
        };
        
        self.status.last_check = Utc::now();
        Ok(self.status.server_status.clone())
    }

    /// Scan for available models on the system
    pub async fn scan_available_models(&mut self) -> OllamaResult<()> {
        debug!("Scanning for available models...");
        
        // List of models to check
        let models_to_check = vec![
            AiModel::QwenTiny,
            AiModel::QwenSmall,
            AiModel::QwenMedium,
            AiModel::QwenCoder,
        ];

        for model in models_to_check {
            let is_available = self.check_model_availability(&model).await;
            self.status.available_models.insert(model.clone(), is_available);
            
            if is_available {
                info!("Model {} is available", model.model_name());
                
                // Initialize performance tracking if not exists
                if !self.performance_tracker.contains_key(&model) {
                    self.performance_tracker.insert(model.clone(), ModelPerformance::new(model.clone()));
                }
            } else {
                debug!("Model {} is not available", model.model_name());
            }
        }

        Ok(())
    }

    /// Check if a specific model is available
    async fn check_model_availability(&mut self, model: &AiModel) -> bool {
        // Try to use the model with a simple test prompt
        let original_model = self.client.get_model().to_string();
        self.client.set_model(model.model_name().to_string());
        
        let test_result = self.client.complete("test").await;
        let is_available = test_result.is_ok();
        
        // Restore original model
        self.client.set_model(original_model);
        
        is_available
    }

    /// Setup the default model (preferring the lightest available model)
    async fn setup_default_model(&mut self) -> OllamaResult<()> {
        // Priority order: start with lightest model
        let model_priority = vec![
            AiModel::QwenTiny,
            AiModel::QwenSmall,
            AiModel::QwenCoder,
            AiModel::QwenMedium,
        ];

        for model in model_priority {
            if *self.status.available_models.get(&model).unwrap_or(&false) {
                self.set_active_model(model).await?;
                info!("Set default model to: {}", self.status.active_model.model_name());
                return Ok(());
            }
        }

        // If no models are available, try to download the tiny model
        warn!("No models available, attempting to download qwen2.5:0.5b");
        self.download_model(&AiModel::QwenTiny).await?;
        self.set_active_model(AiModel::QwenTiny).await?;
        
        Ok(())
    }

    /// Download a model using Ollama
    pub async fn download_model(&mut self, model: &AiModel) -> OllamaResult<()> {
        info!("Downloading model: {}", model.model_name());
        
        // Note: ollama-rs doesn't have a direct download method in the current version
        // We'll need to use the system command or wait for the API to support it
        // For now, we'll provide instructions to the user
        
        let model_name = model.model_name();
        warn!("Model download not yet implemented in ollama-rs");
        warn!("Please run: ollama pull {}", model_name);
        
        // TODO: Implement actual model download when ollama-rs supports it
        // or use system command as fallback
        
        Ok(())
    }

  /// Set the active model for AI operations
    pub async fn set_active_model(&mut self, model: AiModel) -> OllamaResult<()> {
        if !self.status.available_models.get(&model).unwrap_or(&false) {
            return Err(opencircuit_core::OpenCircuitError::AiService(
                format!("Model {} is not available", model.model_name())
            ));
        }

        self.client.set_model(model.model_name().to_string());
        self.status.active_model = model;
        
        info!("Switched to model: {}", self.status.active_model.model_name());
        Ok(())
    }

    /// Get the best model for a specific use case
    pub fn get_best_model_for_use_case(&self, use_case: &AiUseCase) -> Option<AiModel> {
        // Find available models suitable for the use case
        let suitable_models: Vec<AiModel> = self.status.available_models
            .iter()
            .filter(|(model, &available)| available && model.is_suitable_for(use_case))
            .map(|(model, _)| model.clone())
            .collect();

        if suitable_models.is_empty() {
            return None;
        }

        // For now, prefer the model with best performance (lowest response time)
        suitable_models.into_iter()
            .min_by_key(|model| {
                self.performance_tracker
                    .get(model)
                    .map(|perf| perf.avg_response_time_ms)
                    .unwrap_or(u64::MAX)
            })
    }

    /// Auto-select the best model for a use case and switch to it
    pub async fn auto_select_model(&mut self, use_case: &AiUseCase) -> OllamaResult<()> {
        if let Some(best_model) = self.get_best_model_for_use_case(use_case) {
            if best_model != self.status.active_model {
                self.set_active_model(best_model).await?;
            }
        } else {
            warn!("No suitable model found for use case: {:?}", use_case);
        }
        Ok(())
    }

    /// Send a chat message with automatic model selection
    pub async fn chat_with_auto_model(&mut self, message: &str, use_case: &AiUseCase) -> OllamaResult<AiResponse> {
        // Auto-select best model for use case
        self.auto_select_model(use_case).await?;
        
        // Track performance
        let start_time = Instant::now();
        
        // Send the message
        let result = self.client.chat(message).await;
        
        let generation_time_ms = start_time.elapsed().as_millis() as u64;
        
        match result {
            Ok(content) => {
                // Update performance metrics
                self.update_performance_metrics(generation_time_ms, true, None);
                
                let mut response = AiResponse::new(content, self.status.active_model.clone(), generation_time_ms);
                
                // Add contextual follow-up questions based on use case
                self.add_contextual_follow_ups(&mut response, use_case);
                
                Ok(response)
            }
            Err(e) => {
                // Update performance metrics for failure
                self.update_performance_metrics(generation_time_ms, false, None);
                error!("Chat request failed: {}", e);
                Err(e)
            }
        }
    }

    /// Update performance metrics for the current model
    fn update_performance_metrics(&mut self, response_time_ms: u64, success: bool, user_rating: Option<f32>) {
        if let Some(performance) = self.performance_tracker.get_mut(&self.status.active_model) {
            performance.update_metrics(response_time_ms, success, user_rating);
        }
    }

    /// Add contextual follow-up questions based on use case
    fn add_contextual_follow_ups(&self, response: &mut AiResponse, use_case: &AiUseCase) {
        match use_case {
            AiUseCase::BasicChat => {
                response.add_follow_up("Would you like me to help with a specific circuit design?".to_string());
                response.add_follow_up("Do you have any component selection questions?".to_string());
            }
            AiUseCase::ComponentSelection => {
                response.add_follow_up("Would you like me to suggest alternative components?".to_string());
                response.add_follow_up("Do you need help with component specifications?".to_string());
                response.add_follow_up("Should I analyze the power requirements?".to_string());
            }
            AiUseCase::CircuitAnalysis => {
                response.add_follow_up("Would you like me to analyze the frequency response?".to_string());
                response.add_follow_up("Should I check for potential stability issues?".to_string());
                response.add_follow_up("Do you want suggestions for circuit optimization?".to_string());
            }
            AiUseCase::CodeGeneration => {
                response.add_follow_up("Would you like me to generate a SPICE netlist?".to_string());
                response.add_follow_up("Should I create test bench code?".to_string());
                response.add_follow_up("Do you need help with simulation setup?".to_string());
            }
            AiUseCase::ComplexDesign => {
                response.add_follow_up("Would you like me to break this into design phases?".to_string());
                response.add_follow_up("Should I analyze system-level requirements?".to_string());
                response.add_follow_up("Do you want me to suggest a design methodology?".to_string());
            }
        }
    }

    /// Get current status
    pub fn get_status(&self) -> &ModelStatus {
        &self.status
    }

    /// Get performance metrics for all models
    pub fn get_performance_metrics(&self) -> &HashMap<AiModel, ModelPerformance> {
        &self.performance_tracker
    }

    /// Get the current active model
    pub fn get_active_model(&self) -> &AiModel {
        &self.status.active_model
    }

    /// Check if any models are available
    pub fn has_available_models(&self) -> bool {
        self.status.available_models.values().any(|&available| available)
    }

    /// Get list of available models
    pub fn get_available_models(&self) -> Vec<AiModel> {
        self.status.available_models
            .iter()
            .filter(|(_, &available)| available)
            .map(|(model, _)| model.clone())
            .collect()
    }
}

impl Default for OllamaManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_manager_creation() {
        let manager = OllamaManager::new();
        assert_eq!(manager.status.server_status, ServerStatus::Unknown);
        assert!(manager.performance_tracker.is_empty());
    }

    #[test]
    fn test_model_suitability_selection() {
        let manager = OllamaManager::new();
        
        // Test that we can identify suitable models for different use cases
        let basic_chat = AiUseCase::BasicChat;
        let circuit_analysis = AiUseCase::CircuitAnalysis;
        
        // All models should be suitable for basic chat
        assert!(AiModel::QwenTiny.is_suitable_for(&basic_chat));
        assert!(AiModel::QwenMedium.is_suitable_for(&basic_chat));
        
        // Only advanced models should be suitable for circuit analysis
        assert!(!AiModel::QwenTiny.is_suitable_for(&circuit_analysis));
        assert!(AiModel::QwenMedium.is_suitable_for(&circuit_analysis));
    }

    #[test]
    fn test_performance_tracking() {
        let mut manager = OllamaManager::new();
        let model = AiModel::QwenTiny;
        
        // Add performance data
        manager.performance_tracker.insert(model.clone(), ModelPerformance::new(model.clone()));
        manager.update_performance_metrics(1000, true, Some(4.0));
        
        let performance = manager.performance_tracker.get(&model).unwrap();
        assert_eq!(performance.avg_response_time_ms, 1000);
        assert_eq!(performance.success_rate, 1.0);
        assert_eq!(performance.user_rating, 4.0);
    }
}