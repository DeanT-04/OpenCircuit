//! Ollama client for local AI model inference
//! 
//! This module provides a client for interacting with Ollama models locally,
//! offering privacy-focused AI capabilities for circuit design assistance.

use crate::AiResult;
use ollama_rs::Ollama;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Configuration for Ollama client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// Ollama server host
    pub host: String,
    /// Ollama server port
    pub port: u16,
    /// Default model to use
    pub default_model: String,
    /// Maximum conversation history length
    pub max_history: usize,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            host: "http://localhost".to_string(),
            port: 11434,
            default_model: "qwen2.5:0.5b".to_string(),
            max_history: 50,
            timeout_seconds: 30,
        }
    }
}

/// OpenCircuit-specific Ollama client
#[derive(Clone)]
pub struct OpenCircuitOllamaClient {
    /// Ollama client instance
    client: Ollama,
    /// Client configuration
    config: OllamaConfig,
    /// Conversation history
    history: VecDeque<(String, String)>, // (user_message, ai_response)
    /// System prompt for circuit design
    system_prompt: String,
}

impl OpenCircuitOllamaClient {
    /// Create a new Ollama client with default configuration
    pub fn new() -> Self {
        Self::with_config(OllamaConfig::default())
    }

    /// Create a new Ollama client with custom configuration
    pub fn with_config(config: OllamaConfig) -> Self {
        let client = Ollama::new(config.host.clone(), config.port);

        let system_prompt = Self::create_system_prompt();

        Self {
            client,
            config,
            history: VecDeque::new(),
            system_prompt,
        }
    }

    /// Create the system prompt for circuit design assistance
    fn create_system_prompt() -> String {
        r#"You are an expert circuit design assistant for OpenCircuit, a comprehensive EDA tool. Your role is to help users with:

CIRCUIT DESIGN:
- Analog and digital circuit design
- Component selection and specifications
- Circuit analysis and optimization
- Power supply design
- Signal integrity considerations

COMPONENT KNOWLEDGE:
- Resistors, capacitors, inductors
- Operational amplifiers and comparators
- Transistors (BJT, MOSFET, JFET)
- Diodes and voltage regulators
- Microcontrollers and digital ICs
- Connectors and mechanical components

ANALYSIS CAPABILITIES:
- DC operating point analysis
- AC frequency response
- Transient analysis
- Noise analysis
- Thermal considerations
- EMI/EMC compliance

PCB DESIGN:
- Layout guidelines and best practices
- Routing strategies
- Layer stack-up recommendations
- Via placement and sizing
- Ground plane design
- High-speed design considerations

SIMULATION:
- SPICE model recommendations
- Simulation setup and parameters
- Results interpretation
- Design verification methods

Always provide practical, implementable advice with specific part numbers when possible. Focus on cost-effective solutions while maintaining design reliability. Consider manufacturing constraints and real-world component tolerances."#.to_string()
    }

    /// Send a chat message and get response
    pub async fn chat(&mut self, message: &str) -> AiResult<String> {
        // For now, use a simple completion approach
        // This is a simplified implementation that should work with basic ollama-rs
        let full_prompt = format!("{}\n\nUser: {}\nAssistant:", self.system_prompt, message);
        
        match self.client.generate(ollama_rs::generation::completion::request::GenerationRequest::new(
            self.config.default_model.clone(),
            full_prompt,
        )).await {
            Ok(response) => {
                let ai_response = response.response;
                
                // Add to history
                self.add_to_history(message.to_string(), ai_response.clone());
                
                Ok(ai_response)
            }
            Err(e) => {
                Err(opencircuit_core::OpenCircuitError::AiService(
                    format!("Failed to get AI response: {}", e)
                ))
            }
        }
    }

    /// Simple completion without conversation context
    pub async fn complete(&self, prompt: &str) -> AiResult<String> {
        match self.client.generate(ollama_rs::generation::completion::request::GenerationRequest::new(
            self.config.default_model.clone(),
            prompt.to_string(),
        )).await {
            Ok(response) => Ok(response.response),
            Err(e) => Err(opencircuit_core::OpenCircuitError::AiService(
                format!("Failed to complete prompt: {}", e)
            )),
        }
    }

    /// Ask a circuit-specific question with context
    pub async fn ask_circuit_question(&mut self, question: &str, context: Option<&str>) -> AiResult<String> {
        let enhanced_question = match context {
            Some(ctx) => format!("Context: {}\n\nQuestion: {}", ctx, question),
            None => question.to_string(),
        };

        self.chat(&enhanced_question).await
    }

    /// Get component suggestions
    pub async fn suggest_components(&mut self, requirements: &str) -> AiResult<String> {
        let prompt = format!(
            "Based on these requirements, suggest specific electronic components with part numbers:\n\n{}",
            requirements
        );

        self.chat(&prompt).await
    }

    /// Analyze a circuit
    pub async fn analyze_circuit(&mut self, circuit_description: &str) -> AiResult<String> {
        let prompt = format!(
            "Analyze this circuit and provide insights:\n\n{}",
            circuit_description
        );

        self.chat(&prompt).await
    }

    /// Add message pair to conversation history
    fn add_to_history(&mut self, user_message: String, ai_response: String) {
        self.history.push_back((user_message, ai_response));
        
        // Maintain history size limit
        while self.history.len() > self.config.max_history {
            self.history.pop_front();
        }
    }

    /// Clear conversation history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Get conversation history
    pub fn get_history(&self) -> &VecDeque<(String, String)> {
        &self.history
    }

    /// Check if Ollama server is accessible
    pub async fn health_check(&self) -> AiResult<bool> {
        // Simple health check by trying to list models
        match self.client.list_local_models().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// List available models
    pub async fn list_models(&self) -> AiResult<Vec<String>> {
        match self.client.list_local_models().await {
            Ok(models) => {
                let model_names: Vec<String> = models.into_iter()
                    .map(|model| model.name)
                    .collect();
                Ok(model_names)
            }
            Err(e) => Err(opencircuit_core::OpenCircuitError::AiService(
                format!("Failed to list models: {}", e)
            )),
        }
    }

    /// Set the active model
    pub fn set_model(&mut self, model_name: String) {
        self.config.default_model = model_name;
    }

    /// Get current model
    pub fn get_model(&self) -> &str {
        &self.config.default_model
    }

    /// Get configuration
    pub fn get_config(&self) -> &OllamaConfig {
        &self.config
    }
}

impl Default for OpenCircuitOllamaClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = OllamaConfig::default();
        assert_eq!(config.host, "http://localhost");
        assert_eq!(config.port, 11434);
        assert_eq!(config.default_model, "qwen2.5:0.5b");
        assert_eq!(config.max_history, 50);
    }

    #[test]
    fn test_client_creation() {
        let client = OpenCircuitOllamaClient::new();
        assert_eq!(client.get_model(), "qwen2.5:0.5b");
        assert_eq!(client.get_history().len(), 0);
    }

    #[test]
    fn test_system_prompt() {
        let prompt = OpenCircuitOllamaClient::create_system_prompt();
        assert!(prompt.contains("circuit design"));
        assert!(prompt.contains("OpenCircuit"));
    }

    #[test]
    fn test_history_management() {
        let mut client = OpenCircuitOllamaClient::new();
        
        // Test adding to history
        client.add_to_history("Test question".to_string(), "Test response".to_string());
        assert_eq!(client.get_history().len(), 1);
        
        // Test history limit
        for i in 0..60 {
            client.add_to_history(format!("Question {}", i), format!("Response {}", i));
        }
        assert_eq!(client.get_history().len(), 50); // Should be limited to max_history
        
        // Test clearing history
        client.clear_history();
        assert_eq!(client.get_history().len(), 0);
    }
}