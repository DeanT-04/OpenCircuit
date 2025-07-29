# Ollama Rust Integration Guide

## Overview

This guide provides comprehensive examples and patterns for integrating Ollama with OpenCircuit using the `ollama-rs` crate. It covers everything from basic setup to advanced streaming and function calling.

## Dependencies

Add these dependencies to your `Cargo.toml`:

```toml
[dependencies]
ollama-rs = "0.3.1"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
```

For streaming support, enable the stream feature:

```toml
[dependencies]
ollama-rs = { version = "0.3.1", features = ["stream"] }
```

## Basic Client Setup

### Simple Client Initialization

```rust
use ollama_rs::Ollama;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Default connection to localhost:11434
    let ollama = Ollama::default();
    
    // Custom host and port
    let ollama = Ollama::new("http://localhost".to_string(), 11434);
    
    Ok(())
}
```

### Environment-Based Configuration

```rust
use ollama_rs::Ollama;
use std::env;

pub struct OllamaConfig {
    pub host: String,
    pub port: u16,
    pub default_model: String,
}

impl OllamaConfig {
    pub fn from_env() -> Self {
        Self {
            host: env::var("OLLAMA_HOST")
                .unwrap_or_else(|_| "http://localhost".to_string()),
            port: env::var("OLLAMA_PORT")
                .unwrap_or_else(|_| "11434".to_string())
                .parse()
                .unwrap_or(11434),
            default_model: env::var("OLLAMA_MODEL")
                .unwrap_or_else(|_| "qwen2.5:0.5b".to_string()),
        }
    }
    
    pub fn create_client(&self) -> Ollama {
        Ollama::new(self.host.clone(), self.port)
    }
}
```

## Circuit Design Assistant

### Basic Circuit Assistant

```rust
use ollama_rs::{Ollama, generation::completion::GenerationRequest};
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct CircuitQuery {
    pub query_type: QueryType,
    pub content: String,
    pub context: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum QueryType {
    ComponentRecommendation,
    CircuitAnalysis,
    Troubleshooting,
    DesignReview,
    CodeGeneration,
}

pub struct CircuitAssistant {
    ollama: Ollama,
    model: String,
}

impl CircuitAssistant {
    pub fn new(ollama: Ollama, model: String) -> Self {
        Self { ollama, model }
    }
    
    pub async fn ask_circuit_question(&self, query: &CircuitQuery) -> Result<String> {
        let prompt = self.build_circuit_prompt(query);
        
        let request = GenerationRequest::new(self.model.clone(), prompt);
        let response = self.ollama.generate(request).await?;
        
        Ok(response.response)
    }
    
    fn build_circuit_prompt(&self, query: &CircuitQuery) -> String {
        let system_prompt = "You are an expert electronics engineer specializing in circuit design and analysis. Provide clear, practical advice for circuit design questions.";
        
        let context = query.context.as_deref().unwrap_or("");
        
        match query.query_type {
            QueryType::ComponentRecommendation => {
                format!("{}\n\nComponent Recommendation Request:\n{}\n\nContext: {}\n\nProvide specific component recommendations with part numbers, values, and reasoning.", 
                    system_prompt, query.content, context)
            },
            QueryType::CircuitAnalysis => {
                format!("{}\n\nCircuit Analysis Request:\n{}\n\nContext: {}\n\nAnalyze the circuit and explain its operation, potential issues, and improvements.", 
                    system_prompt, query.content, context)
            },
            QueryType::Troubleshooting => {
                format!("{}\n\nTroubleshooting Request:\n{}\n\nContext: {}\n\nDiagnose the problem and provide step-by-step troubleshooting guidance.", 
                    system_prompt, query.content, context)
            },
            QueryType::DesignReview => {
                format!("{}\n\nDesign Review Request:\n{}\n\nContext: {}\n\nReview the design for correctness, efficiency, and best practices.", 
                    system_prompt, query.content, context)
            },
            QueryType::CodeGeneration => {
                format!("{}\n\nCode Generation Request:\n{}\n\nContext: {}\n\nGenerate Rust code for circuit simulation or analysis.", 
                    system_prompt, query.content, context)
            },
        }
    }
}
```

### Usage Example

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let config = OllamaConfig::from_env();
    let ollama = config.create_client();
    let assistant = CircuitAssistant::new(ollama, config.default_model);
    
    // Component recommendation
    let query = CircuitQuery {
        query_type: QueryType::ComponentRecommendation,
        content: "I need a voltage regulator for 5V to 3.3V conversion, 500mA current".to_string(),
        context: Some("Battery-powered IoT device, efficiency is important".to_string()),
    };
    
    let response = assistant.ask_circuit_question(&query).await?;
    println!("Assistant: {}", response);
    
    Ok(())
}
```

## Streaming Responses

### Real-time Circuit Analysis

```rust
use ollama_rs::generation::completion::GenerationRequest;
use tokio_stream::StreamExt;
use tokio::io::{self, AsyncWriteExt};

pub struct StreamingCircuitAssistant {
    ollama: Ollama,
    model: String,
}

impl StreamingCircuitAssistant {
    pub fn new(ollama: Ollama, model: String) -> Self {
        Self { ollama, model }
    }
    
    pub async fn stream_circuit_analysis(&self, circuit_description: &str) -> Result<()> {
        let prompt = format!(
            "Analyze this circuit step by step, explaining each component and its function:\n\n{}",
            circuit_description
        );
        
        let request = GenerationRequest::new(self.model.clone(), prompt);
        let mut stream = self.ollama.generate_stream(request).await?;
        
        let mut stdout = io::stdout();
        print!("Circuit Analysis: ");
        stdout.flush().await?;
        
        while let Some(res) = stream.next().await {
            let responses = res?;
            for resp in responses {
                print!("{}", resp.response);
                stdout.flush().await?;
            }
        }
        
        println!(); // New line after streaming
        Ok(())
    }
}
```

### Streaming with Custom Handler

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct CircuitAnalysisResult {
    pub complete_response: String,
    pub analysis_steps: Vec<String>,
}

impl StreamingCircuitAssistant {
    pub async fn stream_with_collection(&self, prompt: &str) -> Result<CircuitAnalysisResult> {
        let request = GenerationRequest::new(self.model.clone(), prompt.to_string());
        let mut stream = self.ollama.generate_stream(request).await?;
        
        let result = Arc::new(Mutex::new(CircuitAnalysisResult {
            complete_response: String::new(),
            analysis_steps: Vec::new(),
        }));
        
        while let Some(res) = stream.next().await {
            let responses = res?;
            for resp in responses {
                let mut result_guard = result.lock().await;
                result_guard.complete_response.push_str(&resp.response);
                
                // Split into analysis steps (simple example)
                if resp.response.contains('\n') {
                    let lines: Vec<&str> = resp.response.split('\n').collect();
                    for line in lines {
                        if !line.trim().is_empty() {
                            result_guard.analysis_steps.push(line.trim().to_string());
                        }
                    }
                }
            }
        }
        
        let final_result = result.lock().await.clone();
        Ok(final_result)
    }
}

impl Clone for CircuitAnalysisResult {
    fn clone(&self) -> Self {
        Self {
            complete_response: self.complete_response.clone(),
            analysis_steps: self.analysis_steps.clone(),
        }
    }
}
```

## Chat-based Circuit Design

### Circuit Design Chat Session

```rust
use ollama_rs::generation::chat::{ChatMessage, ChatMessageRequest};

pub struct CircuitDesignChat {
    ollama: Ollama,
    model: String,
    history: Vec<ChatMessage>,
}

impl CircuitDesignChat {
    pub fn new(ollama: Ollama, model: String) -> Self {
        let mut history = Vec::new();
        
        // Add system message for circuit design context
        history.push(ChatMessage::system(
            "You are an expert circuit design assistant. Help users design, analyze, and troubleshoot electronic circuits. Provide practical, implementable advice with specific component recommendations and values."
        ));
        
        Self { ollama, model, history }
    }
    
    pub async fn send_message(&mut self, message: &str) -> Result<String> {
        // Add user message to history
        self.history.push(ChatMessage::user(message.to_string()));
        
        let request = ChatMessageRequest::new(
            self.model.clone(),
            vec![ChatMessage::user(message.to_string())]
        );
        
        let response = self.ollama.send_chat_messages_with_history(
            &mut self.history,
            request
        ).await?;
        
        Ok(response.message.content)
    }
    
    pub fn get_conversation_history(&self) -> &[ChatMessage] {
        &self.history
    }
    
    pub fn clear_history(&mut self) {
        self.history.clear();
        // Re-add system message
        self.history.push(ChatMessage::system(
            "You are an expert circuit design assistant. Help users design, analyze, and troubleshoot electronic circuits."
        ));
    }
}
```

### Usage Example

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let config = OllamaConfig::from_env();
    let ollama = config.create_client();
    let mut chat = CircuitDesignChat::new(ollama, config.default_model);
    
    // Start a design conversation
    let response = chat.send_message(
        "I want to design a simple LED driver circuit for a 3.3V microcontroller to drive a 3V LED at 20mA"
    ).await?;
    println!("Assistant: {}", response);
    
    // Follow up question
    let response = chat.send_message(
        "What resistor value should I use and what power rating?"
    ).await?;
    println!("Assistant: {}", response);
    
    Ok(())
}
```

## Model Management

### Model Information and Management

```rust
use ollama_rs::models::LocalModel;

pub struct OllamaModelManager {
    ollama: Ollama,
}

impl OllamaModelManager {
    pub fn new(ollama: Ollama) -> Self {
        Self { ollama }
    }
    
    pub async fn list_available_models(&self) -> Result<Vec<LocalModel>> {
        let models = self.ollama.list_local_models().await?;
        Ok(models)
    }
    
    pub async fn get_model_info(&self, model_name: &str) -> Result<String> {
        let info = self.ollama.show_model_info(model_name.to_string()).await?;
        Ok(format!("Model: {}\nSize: {} bytes\nModified: {:?}", 
            model_name, info.size, info.modified_at))
    }
    
    pub async fn ensure_model_available(&self, model_name: &str) -> Result<bool> {
        let models = self.list_available_models().await?;
        let model_exists = models.iter().any(|m| m.name == model_name);
        
        if !model_exists {
            println!("Model {} not found locally. Please run: ollama pull {}", 
                model_name, model_name);
            return Ok(false);
        }
        
        Ok(true)
    }
}
```

## Advanced Features

### Function Calling for Circuit Tools

```rust
use ollama_rs::coordinator::Coordinator;
use ollama_rs::generation::tools::Tool;
use std::collections::HashMap;

/// Calculate resistor value for LED circuit
#[ollama_rs::function]
async fn calculate_led_resistor(
    supply_voltage: f64,
    led_voltage: f64,
    led_current_ma: f64
) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    let current_a = led_current_ma / 1000.0;
    let voltage_drop = supply_voltage - led_voltage;
    let resistance = voltage_drop / current_a;
    let power = voltage_drop * current_a;
    
    Ok(format!(
        "Resistor value: {:.1}Ω (use {:.1}Ω standard value)\nPower rating: {:.3}W (use {:.2}W resistor)",
        resistance,
        find_standard_resistor(resistance),
        power,
        find_standard_power_rating(power)
    ))
}

/// Find capacitor value for RC filter
#[ollama_rs::function]
async fn calculate_rc_filter(
    resistance_ohms: f64,
    cutoff_frequency_hz: f64
) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    let capacitance = 1.0 / (2.0 * std::f64::consts::PI * resistance_ohms * cutoff_frequency_hz);
    let capacitance_uf = capacitance * 1_000_000.0;
    
    Ok(format!(
        "Capacitor value: {:.2}µF\nCutoff frequency: {:.1}Hz",
        capacitance_uf, cutoff_frequency_hz
    ))
}

fn find_standard_resistor(value: f64) -> f64 {
    // E12 series standard values
    let e12_values = [1.0, 1.2, 1.5, 1.8, 2.2, 2.7, 3.3, 3.9, 4.7, 5.6, 6.8, 8.2];
    let magnitude = 10_f64.powf(value.log10().floor());
    let normalized = value / magnitude;
    
    let closest = e12_values.iter()
        .min_by(|a, b| (normalized - **a).abs().partial_cmp(&(normalized - **b).abs()).unwrap())
        .unwrap();
    
    closest * magnitude
}

fn find_standard_power_rating(power: f64) -> f64 {
    let ratings = [0.125, 0.25, 0.5, 1.0, 2.0, 5.0];
    *ratings.iter()
        .find(|&&rating| rating > power * 2.0) // 2x safety factor
        .unwrap_or(&5.0)
}

pub struct CircuitToolsCoordinator {
    coordinator: Coordinator,
}

impl CircuitToolsCoordinator {
    pub async fn new(ollama: Ollama, model: String) -> Self {
        let coordinator = Coordinator::new(ollama, model, vec![])
            .add_tool(calculate_led_resistor)
            .add_tool(calculate_rc_filter);
        
        Self { coordinator }
    }
    
    pub async fn solve_circuit_problem(&mut self, problem: &str) -> Result<String> {
        let messages = vec![ChatMessage::user(problem.to_string())];
        let response = self.coordinator.chat(messages).await?;
        Ok(response.message.content)
    }
}
```

### Usage with Function Calling

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let config = OllamaConfig::from_env();
    let ollama = config.create_client();
    let mut coordinator = CircuitToolsCoordinator::new(ollama, "qwen2.5:1b".to_string()).await;
    
    let response = coordinator.solve_circuit_problem(
        "I need to drive an LED with 2.1V forward voltage at 20mA from a 5V supply. What resistor should I use?"
    ).await?;
    
    println!("Solution: {}", response);
    
    Ok(())
}
```

## Error Handling and Resilience

### Robust Circuit Assistant

```rust
use std::time::Duration;
use tokio::time::timeout;

pub struct RobustCircuitAssistant {
    ollama: Ollama,
    model: String,
    fallback_model: Option<String>,
    timeout_duration: Duration,
}

impl RobustCircuitAssistant {
    pub fn new(ollama: Ollama, model: String) -> Self {
        Self {
            ollama,
            model,
            fallback_model: Some("qwen2.5:0.5b".to_string()),
            timeout_duration: Duration::from_secs(30),
        }
    }
    
    pub async fn ask_with_fallback(&self, question: &str) -> Result<String> {
        // Try primary model first
        match self.try_ask(&self.model, question).await {
            Ok(response) => Ok(response),
            Err(e) => {
                println!("Primary model failed: {}. Trying fallback...", e);
                
                if let Some(fallback) = &self.fallback_model {
                    self.try_ask(fallback, question).await
                } else {
                    Err(e)
                }
            }
        }
    }
    
    async fn try_ask(&self, model: &str, question: &str) -> Result<String> {
        let request = GenerationRequest::new(model.to_string(), question.to_string());
        
        let response = timeout(
            self.timeout_duration,
            self.ollama.generate(request)
        ).await??;
        
        Ok(response.response)
    }
}
```

## Testing and Validation

### Circuit Knowledge Validation

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_basic_circuit_knowledge() {
        let config = OllamaConfig::from_env();
        let ollama = config.create_client();
        let assistant = CircuitAssistant::new(ollama, config.default_model);
        
        let query = CircuitQuery {
            query_type: QueryType::ComponentRecommendation,
            content: "What is Ohm's law?".to_string(),
            context: None,
        };
        
        let response = assistant.ask_circuit_question(&query).await.unwrap();
        assert!(response.to_lowercase().contains("voltage"));
        assert!(response.to_lowercase().contains("current"));
        assert!(response.to_lowercase().contains("resistance"));
    }
    
    #[tokio::test]
    async fn test_model_availability() {
        let config = OllamaConfig::from_env();
        let ollama = config.create_client();
        let manager = OllamaModelManager::new(ollama);
        
        let available = manager.ensure_model_available(&config.default_model).await.unwrap();
        assert!(available, "Default model should be available for testing");
    }
}
```

## Performance Optimization

### Connection Pooling and Caching

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

pub struct CachedCircuitAssistant {
    ollama: Ollama,
    model: String,
    cache: Arc<RwLock<HashMap<String, String>>>,
}

impl CachedCircuitAssistant {
    pub fn new(ollama: Ollama, model: String) -> Self {
        Self {
            ollama,
            model,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn ask_with_cache(&self, question: &str) -> Result<String> {
        let cache_key = format!("{}:{}", self.model, question);
        
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached_response) = cache.get(&cache_key) {
                return Ok(cached_response.clone());
            }
        }
        
        // Generate new response
        let request = GenerationRequest::new(self.model.clone(), question.to_string());
        let response = self.ollama.generate(request).await?;
        
        // Cache the response
        {
            let mut cache = self.cache.write().await;
            cache.insert(cache_key, response.response.clone());
        }
        
        Ok(response.response)
    }
}
```

This comprehensive guide provides the foundation for integrating Ollama with OpenCircuit using Rust. Start with the basic examples and gradually incorporate more advanced features as your application grows.