//! AI models and data structures for OpenCircuit
//! 
//! This module defines the data structures used for AI interactions,
//! model management, and circuit design assistance.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fmt;

/// Available AI models for different use cases
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AiModel {
    /// Ultra-lightweight model for testing and basic interactions (0.5B parameters)
    QwenTiny,
    /// Balanced model for production use (1B parameters)
    QwenSmall,
    /// Advanced model for complex circuit analysis (3B parameters)
    QwenMedium,
    /// Specialized coding model for circuit generation
    QwenCoder,
    /// Custom model specified by user
    Custom(String),
}

impl AiModel {
    /// Get the Ollama model name string
    pub fn model_name(&self) -> &str {
        match self {
            AiModel::QwenTiny => "qwen2.5:0.5b",
            AiModel::QwenSmall => "qwen2.5:1b",
            AiModel::QwenMedium => "qwen2.5:3b",
            AiModel::QwenCoder => "qwen2.5-coder:1.5b",
            AiModel::Custom(name) => name,
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &str {
        match self {
            AiModel::QwenTiny => "Ultra-lightweight (0.5B) - Fast responses, basic assistance",
            AiModel::QwenSmall => "Balanced (1B) - Good performance, general circuit design",
            AiModel::QwenMedium => "Advanced (3B) - Complex analysis, detailed explanations",
            AiModel::QwenCoder => "Coding specialist (1.5B) - Circuit generation, code assistance",
            AiModel::Custom(name) => name,
        }
    }

    /// Get estimated memory usage in GB
    pub fn memory_usage_gb(&self) -> f32 {
        match self {
            AiModel::QwenTiny => 0.5,
            AiModel::QwenSmall => 1.0,
            AiModel::QwenMedium => 2.5,
            AiModel::QwenCoder => 1.5,
            AiModel::Custom(_) => 1.0, // Default estimate
        }
    }

    /// Check if model is suitable for the given use case
    pub fn is_suitable_for(&self, use_case: &AiUseCase) -> bool {
        match use_case {
            AiUseCase::BasicChat => true, // All models can handle basic chat
            AiUseCase::ComponentSelection => matches!(self, AiModel::QwenSmall | AiModel::QwenMedium | AiModel::QwenCoder),
            AiUseCase::CircuitAnalysis => matches!(self, AiModel::QwenMedium | AiModel::QwenCoder),
            AiUseCase::CodeGeneration => matches!(self, AiModel::QwenCoder | AiModel::QwenMedium),
            AiUseCase::ComplexDesign => matches!(self, AiModel::QwenMedium),
        }
    }
}

impl Default for AiModel {
    fn default() -> Self {
        AiModel::QwenTiny // Start with the lightest model
    }
}

impl fmt::Display for AiModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.model_name())
    }
}

/// Different use cases for AI assistance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AiUseCase {
    /// Basic chat and simple questions
    BasicChat,
    /// Component selection and recommendations
    ComponentSelection,
    /// Circuit analysis and optimization
    CircuitAnalysis,
    /// Code and netlist generation
    CodeGeneration,
    /// Complex multi-stage design projects
    ComplexDesign,
}

/// Model performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformance {
    /// Model identifier
    pub model: AiModel,
    /// Average response time in milliseconds
    pub avg_response_time_ms: u64,
    /// Memory usage in MB
    pub memory_usage_mb: u64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f32,
    /// User satisfaction rating (1-5)
    pub user_rating: f32,
    /// Number of interactions recorded
    pub interaction_count: u32,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

impl ModelPerformance {
    pub fn new(model: AiModel) -> Self {
        Self {
            model,
            avg_response_time_ms: 0,
            memory_usage_mb: 0,
            success_rate: 0.0,
            user_rating: 0.0,
            interaction_count: 0,
            last_updated: Utc::now(),
        }
    }

    /// Update performance metrics with new interaction data
    pub fn update_metrics(&mut self, response_time_ms: u64, success: bool, user_rating: Option<f32>) {
        // Update average response time
        let total_time = self.avg_response_time_ms * self.interaction_count as u64;
        self.interaction_count += 1;
        self.avg_response_time_ms = (total_time + response_time_ms) / self.interaction_count as u64;

        // Update success rate
        let total_successes = (self.success_rate * (self.interaction_count - 1) as f32) + if success { 1.0 } else { 0.0 };
        self.success_rate = total_successes / self.interaction_count as f32;

        // Update user rating if provided
        if let Some(rating) = user_rating {
            let total_rating = self.user_rating * (self.interaction_count - 1) as f32;
            self.user_rating = (total_rating + rating) / self.interaction_count as f32;
        }

        self.last_updated = Utc::now();
    }
}

/// AI interaction context for better responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiContext {
    /// Current project or circuit being worked on
    pub project_name: Option<String>,
    /// Circuit type (analog, digital, mixed-signal, power, RF)
    pub circuit_type: Option<CircuitType>,
    /// Design constraints and requirements
    pub constraints: Vec<String>,
    /// Previously discussed components
    pub mentioned_components: Vec<String>,
    /// Current design phase
    pub design_phase: DesignPhase,
    /// User expertise level
    pub user_level: ExpertiseLevel,
}

impl Default for AiContext {
    fn default() -> Self {
        Self {
            project_name: None,
            circuit_type: None,
            constraints: Vec::new(),
            mentioned_components: Vec::new(),
            design_phase: DesignPhase::Conceptual,
            user_level: ExpertiseLevel::Intermediate,
        }
    }
}

/// Circuit type categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CircuitType {
    Analog,
    Digital,
    MixedSignal,
    Power,
    RF,
    Audio,
    Sensor,
    Motor,
    Communication,
    Other(String),
}

/// Design phase in the development process
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DesignPhase {
    /// Initial concept and requirements
    Conceptual,
    /// Schematic design
    Schematic,
    /// Component selection
    ComponentSelection,
    /// Simulation and analysis
    Simulation,
    /// PCB layout
    Layout,
    /// Testing and validation
    Testing,
    /// Production preparation
    Production,
}

/// User expertise level for tailored responses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExpertiseLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// AI response with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    /// The actual response content
    pub content: String,
    /// Model used to generate the response
    pub model: AiModel,
    /// Response generation time in milliseconds
    pub generation_time_ms: u64,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Suggested follow-up questions
    pub follow_up_questions: Vec<String>,
    /// Referenced components or concepts
    pub references: Vec<String>,
    /// Timestamp of response
    pub timestamp: DateTime<Utc>,
}

impl AiResponse {
    pub fn new(content: String, model: AiModel, generation_time_ms: u64) -> Self {
        Self {
            content,
            model,
            generation_time_ms,
            confidence: 0.8, // Default confidence
            follow_up_questions: Vec::new(),
            references: Vec::new(),
            timestamp: Utc::now(),
        }
    }

    /// Add a follow-up question suggestion
    pub fn add_follow_up(&mut self, question: String) {
        self.follow_up_questions.push(question);
    }

    /// Add a reference (component, concept, etc.)
    pub fn add_reference(&mut self, reference: String) {
        self.references.push(reference);
    }
}

/// Model management status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatus {
    /// Available models on the system
    pub available_models: HashMap<AiModel, bool>,
    /// Currently active model
    pub active_model: AiModel,
    /// Model performance history
    pub performance_history: HashMap<AiModel, ModelPerformance>,
    /// Ollama server status
    pub server_status: ServerStatus,
    /// Last status check
    pub last_check: DateTime<Utc>,
}

/// Ollama server status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerStatus {
    /// Server is running and responsive
    Running,
    /// Server is not running
    Stopped,
    /// Server status unknown or unreachable
    Unknown,
    /// Server is starting up
    Starting,
}

impl Default for ModelStatus {
    fn default() -> Self {
        Self {
            available_models: HashMap::new(),
            active_model: AiModel::default(),
            performance_history: HashMap::new(),
            server_status: ServerStatus::Unknown,
            last_check: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_model_names() {
        assert_eq!(AiModel::QwenTiny.model_name(), "qwen2.5:0.5b");
        assert_eq!(AiModel::QwenSmall.model_name(), "qwen2.5:1b");
        assert_eq!(AiModel::QwenMedium.model_name(), "qwen2.5:3b");
        assert_eq!(AiModel::QwenCoder.model_name(), "qwen2.5-coder:1.5b");
    }

    #[test]
    fn test_model_suitability() {
        let tiny = AiModel::QwenTiny;
        let medium = AiModel::QwenMedium;
        
        assert!(tiny.is_suitable_for(&AiUseCase::BasicChat));
        assert!(!tiny.is_suitable_for(&AiUseCase::CircuitAnalysis));
        assert!(medium.is_suitable_for(&AiUseCase::CircuitAnalysis));
        assert!(medium.is_suitable_for(&AiUseCase::ComplexDesign));
    }

    #[test]
    fn test_performance_metrics_update() {
        let mut perf = ModelPerformance::new(AiModel::QwenTiny);
        
        perf.update_metrics(1000, true, Some(4.0));
        assert_eq!(perf.avg_response_time_ms, 1000);
        assert_eq!(perf.success_rate, 1.0);
        assert_eq!(perf.user_rating, 4.0);
        assert_eq!(perf.interaction_count, 1);
        
        perf.update_metrics(2000, false, Some(3.0));
        assert_eq!(perf.avg_response_time_ms, 1500);
        assert_eq!(perf.success_rate, 0.5);
        assert_eq!(perf.user_rating, 3.5);
        assert_eq!(perf.interaction_count, 2);
    }

    #[test]
    fn test_ai_response_creation() {
        let response = AiResponse::new(
            "Test response".to_string(),
            AiModel::QwenTiny,
            500
        );
        
        assert_eq!(response.content, "Test response");
        assert_eq!(response.model, AiModel::QwenTiny);
        assert_eq!(response.generation_time_ms, 500);
        assert_eq!(response.confidence, 0.8);
    }
}