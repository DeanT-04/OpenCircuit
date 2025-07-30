//! AI integration module for OpenCircuit
//! 
//! This module contains:
//! - Chat handler for AI conversations
//! - Ollama client for local AI model inference
//! - Model management and automatic selection
//! - Component recommendation system
//! - Vector embeddings for component search

pub mod chat_handler;
pub mod ollama_client;
pub mod models;
pub mod ollama_manager;
pub mod component_advisor;
pub mod embeddings;
pub mod circuit_generator;
pub mod circuit_simulator;
pub mod docs;

use anyhow::Result;
use tracing::{info, warn, error};

use crate::models::ModelStatus;
use opencircuit_core::OpenCircuitError;

// Type alias for AI-specific results
pub type AiResult<T> = Result<T, OpenCircuitError>;

/// AI service configuration
#[derive(Debug, Clone)]
pub struct AiConfig {
    /// Ollama server host
    pub ollama_host: String,
    /// Ollama server port
    pub ollama_port: u16,
    /// Default model to use
    pub default_model: models::AiModel,
    /// Maximum conversation history length
    pub max_history: usize,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            ollama_host: "http://localhost".to_string(),
            ollama_port: 11434,
            default_model: models::AiModel::QwenTiny,
            max_history: 50,
            timeout_seconds: 30,
        }
    }
}

/// Main AI service for OpenCircuit
pub struct AiService {
    /// Ollama model manager
    manager: ollama_manager::OllamaManager,
    /// Service configuration
    config: AiConfig,
    /// Component recommendation advisor
    component_advisor: component_advisor::ComponentAdvisor,
    /// Component embedding engine for similarity search
    embedding_engine: embeddings::ComponentEmbeddingEngine,
}

impl AiService {
    /// Create a new AI service with default configuration
    pub async fn new() -> AiResult<Self> {
        Self::with_config(AiConfig::default()).await
    }

    /// Create a new AI service with custom configuration
    pub async fn with_config(config: AiConfig) -> AiResult<Self> {
        let ollama_config = ollama_client::OllamaConfig {
            host: config.ollama_host.clone(),
            port: config.ollama_port,
            default_model: config.default_model.model_name().to_string(),
            max_history: config.max_history,
            timeout_seconds: config.timeout_seconds,
        };

        let manager = ollama_manager::OllamaManager::with_config(ollama_config.clone());
        let ollama_client = ollama_client::OpenCircuitOllamaClient::with_config(ollama_config.clone());
        let component_advisor = component_advisor::ComponentAdvisor::new(ollama_client.clone()).await?;
        let embedding_engine = embeddings::ComponentEmbeddingEngine::new(ollama_client.clone()).await?;

        Ok(Self { manager, config, component_advisor, embedding_engine })
    }

    /// Initialize the AI service
    pub async fn initialize(&mut self) -> AiResult<()> {
        self.manager.initialize().await
    }

    /// Send a chat message with automatic model selection
    pub async fn chat(&mut self, message: &str, use_case: models::AiUseCase) -> AiResult<models::AiResponse> {
        self.manager.chat_with_auto_model(message, &use_case).await
    }

    /// Ask a circuit-specific question
    pub async fn ask_circuit_question(&mut self, question: &str, circuit_context: Option<&str>) -> AiResult<models::AiResponse> {
        // Determine use case based on question content
        let use_case = self.determine_use_case(question);
        
        let enhanced_question = match circuit_context {
            Some(context) => format!("Circuit Context: {}\n\nQuestion: {}", context, question),
            None => question.to_string(),
        };

        self.chat(&enhanced_question, use_case).await
    }

    /// Get component recommendations
    pub async fn suggest_components(&mut self, request: component_advisor::RecommendationRequest) -> AiResult<Vec<component_advisor::ComponentRecommendation>> {
        self.component_advisor.get_recommendations(request).await
    }

    /// Find similar components using vector embeddings
    pub async fn find_similar_components(
        &mut self,
        component: &opencircuit_core::models::Component,
        limit: usize,
    ) -> AiResult<Vec<embeddings::SimilarityMatch>> {
        self.embedding_engine.find_similar_components(component, limit).await
    }

    /// Analyze a circuit
    pub async fn analyze_circuit(&mut self, circuit_description: &str) -> AiResult<models::AiResponse> {
        let prompt = format!(
            "Analyze this circuit and provide insights on performance, potential issues, and improvements:\n\n{}",
            circuit_description
        );

        self.chat(&prompt, models::AiUseCase::CircuitAnalysis).await
    }

    /// Generate circuit code or netlist
    pub async fn generate_circuit_code(&mut self, requirements: &str, format: &str) -> AiResult<models::AiResponse> {
        let prompt = format!(
            "Generate {} code for a circuit with these requirements:\n\n{}",
            format, requirements
        );

        self.chat(&prompt, models::AiUseCase::CodeGeneration).await
    }

    /// Determine the appropriate use case based on the question content
    fn determine_use_case(&self, question: &str) -> models::AiUseCase {
        let question_lower = question.to_lowercase();

        // Check for code generation first (most specific)
        if question_lower.contains("generate") || question_lower.contains("code") ||
           question_lower.contains("netlist") || question_lower.contains("spice") {
            models::AiUseCase::CodeGeneration
        } else if question_lower.contains("analyze") || question_lower.contains("performance") ||
                  question_lower.contains("frequency") || question_lower.contains("stability") {
            models::AiUseCase::CircuitAnalysis
        } else if question_lower.contains("component") || question_lower.contains("part") || 
                  question_lower.contains("resistor") || question_lower.contains("capacitor") ||
                  question_lower.contains("ic") || question_lower.contains("transistor") {
            models::AiUseCase::ComponentSelection
        } else if question_lower.contains("design") && 
                  (question_lower.contains("complex") || question_lower.contains("system")) {
            models::AiUseCase::ComplexDesign
        } else {
            models::AiUseCase::BasicChat
        }
    }

    /// Get current AI service status
    pub fn get_status(&self) -> &ModelStatus {
        self.manager.get_status()
    }

    /// Check if AI service is ready
    pub fn is_ready(&self) -> bool {
        self.manager.has_available_models()
    }

    /// Get available models
    pub fn get_available_models(&self) -> Vec<models::AiModel> {
        self.manager.get_available_models()
    }

    /// Set active model manually
    pub async fn set_model(&mut self, model: models::AiModel) -> AiResult<()> {
        self.manager.set_active_model(model).await
    }

    /// Get current active model
    pub fn get_active_model(&self) -> &models::AiModel {
        self.manager.get_active_model()
    }

    /// Legacy method for backward compatibility
    pub async fn chat_completion(&self, prompt: &str) -> AiResult<String> {
        // This is a simplified version for backward compatibility
        // In practice, you'd want to use the new chat method
        Ok(format!("AI response to: {}", prompt))
    }
}

// Re-export important types for easy access
pub use chat_handler::ChatHandler;
pub use ollama_client::OpenCircuitOllamaClient;
pub use models::{
    AiContext, CircuitType, DesignPhase, ExpertiseLevel, AiResponse, 
    AiModel, AiUseCase, ModelPerformance, ServerStatus
};
pub use component_advisor::{
    ComponentAdvisor, ComponentRecommendation, RecommendationRequest,
    BudgetConstraints, PerformancePriority, CostCategory, CompatibilityAnalysis
};
pub use embeddings::{
    ComponentEmbeddingEngine, ComponentEmbedding, SimilarityMatch
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AiModel, AiUseCase};

    #[test]
    fn test_ai_config_default() {
        let config = AiConfig::default();
        assert_eq!(config.ollama_host, "http://localhost");
        assert_eq!(config.ollama_port, 11434);
        assert_eq!(config.default_model, AiModel::QwenTiny);
        assert_eq!(config.max_history, 50);
    }

    #[test]
    fn test_use_case_determination() {
        // Test the use case determination logic without creating a full service
        let test_cases = vec![
            ("What resistor should I use?", models::AiUseCase::ComponentSelection),
            ("Analyze this amplifier circuit", models::AiUseCase::CircuitAnalysis),
            ("Generate SPICE netlist", models::AiUseCase::CodeGeneration),
            ("Hello, how are you?", models::AiUseCase::BasicChat),
        ];

        for (question, expected) in test_cases {
            let question_lower = question.to_lowercase();
            let actual = if question_lower.contains("generate") || question_lower.contains("code") ||
                           question_lower.contains("netlist") || question_lower.contains("spice") {
                models::AiUseCase::CodeGeneration
            } else if question_lower.contains("analyze") || question_lower.contains("performance") ||
                      question_lower.contains("frequency") || question_lower.contains("stability") {
                models::AiUseCase::CircuitAnalysis
            } else if question_lower.contains("component") || question_lower.contains("part") || 
                      question_lower.contains("resistor") || question_lower.contains("capacitor") ||
                      question_lower.contains("ic") || question_lower.contains("transistor") {
                models::AiUseCase::ComponentSelection
            } else if question_lower.contains("design") && 
                      (question_lower.contains("complex") || question_lower.contains("system")) {
                models::AiUseCase::ComplexDesign
            } else {
                models::AiUseCase::BasicChat
            };
            
            assert_eq!(actual, expected, "Failed for question: {}", question);
        }
    }

    #[tokio::test]
    async fn test_ai_service_creation() {
        let service = AiService::new().await;
        assert!(service.is_ok());
        if let Ok(service) = service {
            assert_eq!(service.config.default_model, AiModel::QwenTiny);
        }
    }
}