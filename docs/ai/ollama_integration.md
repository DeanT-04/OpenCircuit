---
title: Ollama Integration Guide
description: Complete guide for integrating Ollama local AI models with OpenCircuit
last_updated: 2025-01-27
tags: [ai, ollama, local-models, rust, integration]
context_id: ai.ollama.integration
---

# 🦙 Ollama Integration Guide

OpenCircuit leverages Ollama for local AI model inference, providing privacy-focused, offline-capable AI assistance for circuit design without relying on external APIs.

## 🏗️ Ollama Architecture Overview

```
┌─────────────────────────────────────┐
│         OpenCircuit Frontend       │
├─────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐   │
│  │ AI Assistant│  │ Chat Panel  │   │
│  │   (Local)   │  │   (egui)    │   │
│  └─────────────┘  └─────────────┘   │
├─────────────────────────────────────┤
│         Ollama Client (Rust)       │
│      (ollama-rs integration)       │
├─────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐   │
│  │ Ollama API  │  │ Local Models│   │
│  │ (HTTP/REST) │  │ (GGUF/GGML) │   │
│  └─────────────┘  └─────────────┘   │
└─────────────────────────────────────┘
```

## 🚀 Quick Start

### 1. Install Ollama

**Windows:**
```powershell
# Download and install from https://ollama.ai/download
# Or use winget
winget install Ollama.Ollama
```

**Linux/macOS:**
```bash
curl -fsSL https://ollama.ai/install.sh | sh
```

### 2. Pull Recommended Models

```bash
# Small, fast model for testing (0.5B parameters)
ollama pull qwen2.5:0.5b

# Balanced model for production (1B parameters)
ollama pull qwen2.5:1b

# Advanced model for complex tasks (3B parameters)
ollama pull qwen2.5:3b

# Specialized coding model
ollama pull qwen2.5-coder:1.5b
```

### 3. Start Ollama Server

```bash
# Start Ollama server (default: localhost:11434)
ollama serve

# Or run in background
ollama serve &
```

## 🦀 Rust Integration

### Dependencies

Add to `Cargo.toml`:
```toml
[dependencies]
ollama-rs = "0.3.1"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
```

### Basic Client Setup

```rust
// @context_id: ai.ollama.client_setup
// @purpose: Initialize Ollama client for OpenCircuit
use ollama_rs::Ollama;
use ollama_rs::generation::completion::GenerationRequest;
use ollama_rs::generation::chat::{ChatMessage, ChatMessageRequest};

pub struct OpenCircuitOllamaClient {
    client: Ollama,
    model: String,
    conversation_history: Vec<ChatMessage>,
}

impl OpenCircuitOllamaClient {
    pub fn new(host: Option<String>, port: Option<u16>, model: String) -> Self {
        let client = match (host, port) {
            (Some(h), Some(p)) => Ollama::new(h, p),
            _ => Ollama::default(), // localhost:11434
        };
        
        Self {
            client,
            model,
            conversation_history: Vec::new(),
        }
    }
    
    pub async fn chat(&mut self, message: &str) -> anyhow::Result<String> {
        // Add user message to history
        self.conversation_history.push(ChatMessage::user(message.to_string()));
        
        // Send chat request with history
        let response = self.client
            .send_chat_messages_with_history(
                &mut self.conversation_history,
                ChatMessageRequest::new(
                    self.model.clone(),
                    vec![ChatMessage::user(message.to_string())],
                ),
            )
            .await?;
        
        Ok(response.message.content)
    }
}
```

### Circuit Design Assistant

```rust
// @context_id: ai.ollama.circuit_assistant
// @purpose: Specialized circuit design assistant using Ollama
use ollama_rs::generation::chat::ChatMessage;

pub struct CircuitDesignAssistant {
    ollama_client: OpenCircuitOllamaClient,
    system_prompt: String,
}

impl CircuitDesignAssistant {
    pub fn new(model: String) -> Self {
        let system_prompt = r#"
You are an expert electronics engineer and circuit design assistant. You help users:
1. Design electronic circuits from requirements
2. Select appropriate components (resistors, capacitors, ICs, etc.)
3. Analyze circuit behavior and performance
4. Suggest PCB layout best practices
5. Troubleshoot circuit issues
6. Explain electronic concepts clearly

Always provide practical, implementable advice with specific component values and part numbers when possible.
Focus on cost-effective, readily available components.
Consider power consumption, thermal management, and manufacturability.
"#.trim().to_string();

        let mut client = OpenCircuitOllamaClient::new(None, None, model);
        
        // Initialize with system prompt
        client.conversation_history.push(ChatMessage::system(system_prompt.clone()));
        
        Self {
            ollama_client: client,
            system_prompt,
        }
    }
    
    pub async fn ask_circuit_question(&mut self, question: &str, circuit_context: Option<&str>) -> anyhow::Result<String> {
        let enhanced_question = match circuit_context {
            Some(context) => format!("Circuit Context: {}\n\nQuestion: {}", context, question),
            None => question.to_string(),
        };
        
        self.ollama_client.chat(&enhanced_question).await
    }
    
    pub async fn suggest_components(&mut self, requirements: &str) -> anyhow::Result<String> {
        let prompt = format!(
            "Based on these requirements, suggest specific electronic components with part numbers:\n\n{}",
            requirements
        );
        
        self.ollama_client.chat(&prompt).await
    }
    
    pub async fn analyze_circuit(&mut self, netlist: &str) -> anyhow::Result<String> {
        let prompt = format!(
            "Analyze this circuit netlist and provide insights on performance, potential issues, and improvements:\n\n{}",
            netlist
        );
        
        self.ollama_client.chat(&prompt).await
    }
}
```

### Streaming Responses

```rust
// @context_id: ai.ollama.streaming
// @purpose: Real-time streaming responses for better UX
use ollama_rs::generation::completion::GenerationRequest;
use tokio_stream::StreamExt;

impl OpenCircuitOllamaClient {
    pub async fn chat_stream(&self, message: &str) -> anyhow::Result<impl StreamExt<Item = Result<String, anyhow::Error>>> {
        let request = GenerationRequest::new(self.model.clone(), message.to_string());
        
        let stream = self.client.generate_stream(request).await?;
        
        Ok(stream.map(|result| {
            result
                .map_err(|e| anyhow::anyhow!("Stream error: {}", e))
                .map(|responses| {
                    responses
                        .into_iter()
                        .map(|resp| resp.response)
                        .collect::<Vec<_>>()
                        .join("")
                })
        }))
    }
}
```

## 🎯 Model Selection Guide

### Recommended Models for OpenCircuit

#### 1. **Qwen 2.5 Series** (Primary Choice)
```bash
# Ultra-lightweight for testing
ollama pull qwen2.5:0.5b    # 0.5B params, ~400MB

# Balanced performance
ollama pull qwen2.5:1b      # 1B params, ~800MB
ollama pull qwen2.5:3b      # 3B params, ~2GB

# High performance (if resources allow)
ollama pull qwen2.5:7b      # 7B params, ~4.5GB
```

**Pros:**
- Apache 2.0 license (fully open source)
- Excellent reasoning capabilities
- Strong coding performance
- Efficient inference
- Good instruction following

#### 2. **Qwen 2.5-Coder** (Specialized)
```bash
ollama pull qwen2.5-coder:1.5b  # 1.5B params, optimized for code
ollama pull qwen2.5-coder:3b    # 3B params, enhanced coding
```

**Use Cases:**
- Circuit netlist generation
- PCB layout code
- Simulation script creation
- Component library code

#### 3. **Gemma 2 Series** (Alternative)
```bash
ollama pull gemma2:2b       # 2B params, ~1.5GB
ollama pull gemma2:9b       # 9B params, ~5.5GB
```

**Pros:**
- Google's open model
- Strong performance
- Good safety alignment
- Multimodal capabilities (future)

### Performance Comparison

| Model | Size | RAM Usage | Speed | Quality | Use Case |
|-------|------|-----------|-------|---------|----------|
| qwen2.5:0.5b | 400MB | 1GB | Very Fast | Good | Testing, Quick responses |
| qwen2.5:1b | 800MB | 2GB | Fast | Very Good | Production, Balanced |
| qwen2.5:3b | 2GB | 4GB | Medium | Excellent | Complex analysis |
| qwen2.5-coder:1.5b | 1.2GB | 3GB | Fast | Excellent (Code) | Code generation |
| gemma2:2b | 1.5GB | 3GB | Fast | Very Good | Alternative choice |

## ⚙️ Configuration

### Environment Variables

```bash
# Ollama server configuration
export OLLAMA_HOST=0.0.0.0:11434
export OLLAMA_KEEP_ALIVE=5m
export OLLAMA_MAX_LOADED_MODELS=3
export OLLAMA_NUM_PARALLEL=4
export OLLAMA_MAX_QUEUE=512

# OpenCircuit specific
export OPENCIRCUIT_OLLAMA_MODEL=qwen2.5:1b
export OPENCIRCUIT_OLLAMA_HOST=localhost:11434
export OPENCIRCUIT_AI_TIMEOUT=30
```

### Rust Configuration

```rust
// @context_id: ai.ollama.config
// @purpose: Configuration management for Ollama integration
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub host: String,
    pub port: u16,
    pub model: String,
    pub timeout_seconds: u64,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub keep_alive: Option<String>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 11434,
            model: "qwen2.5:1b".to_string(),
            timeout_seconds: 30,
            max_tokens: Some(2048),
            temperature: Some(0.7),
            keep_alive: Some("5m".to_string()),
        }
    }
}

impl OllamaConfig {
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("OPENCIRCUIT_OLLAMA_HOST")
                .unwrap_or_else(|_| "localhost".to_string())
                .split(':')
                .next()
                .unwrap_or("localhost")
                .to_string(),
            port: std::env::var("OPENCIRCUIT_OLLAMA_HOST")
                .unwrap_or_else(|_| "localhost:11434".to_string())
                .split(':')
                .nth(1)
                .and_then(|p| p.parse().ok())
                .unwrap_or(11434),
            model: std::env::var("OPENCIRCUIT_OLLAMA_MODEL")
                .unwrap_or_else(|_| "qwen2.5:1b".to_string()),
            timeout_seconds: std::env::var("OPENCIRCUIT_AI_TIMEOUT")
                .and_then(|t| t.parse().ok())
                .unwrap_or(30),
            ..Default::default()
        }
    }
}
```

## 🔧 Advanced Features

### Function Calling with Tools

```rust
// @context_id: ai.ollama.function_calling
// @purpose: Tool integration for enhanced circuit design capabilities
use ollama_rs::coordinator::Coordinator;
use ollama_rs::generation::tools::implementations::{Calculator};

pub struct CircuitCalculator;

#[ollama_rs::function]
async fn calculate_resistor_divider(
    vin: f64,
    vout: f64,
    r1: Option<f64>
) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    let ratio = vout / vin;
    
    match r1 {
        Some(r1_val) => {
            let r2 = r1_val * ratio / (1.0 - ratio);
            Ok(format!("For Vin={:.2}V, Vout={:.2}V, R1={:.0}Ω: R2={:.0}Ω", vin, vout, r1_val, r2))
        }
        None => {
            Ok(format!("Voltage ratio: {:.3}, choose R1 then R2 = R1 * {:.3}", ratio, ratio / (1.0 - ratio)))
        }
    }
}

#[ollama_rs::function]
async fn calculate_rc_time_constant(
    resistance: f64,
    capacitance: f64
) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    let tau = resistance * capacitance;
    Ok(format!("RC time constant: {:.6} seconds ({:.3} ms)", tau, tau * 1000.0))
}

impl CircuitDesignAssistant {
    pub async fn create_with_tools(model: String) -> anyhow::Result<Self> {
        let ollama = Ollama::default();
        let mut coordinator = Coordinator::new(ollama, model, vec![])
            .add_tool(Calculator {});
        
        // Add custom circuit tools
        // coordinator = coordinator.add_tool(calculate_resistor_divider);
        // coordinator = coordinator.add_tool(calculate_rc_time_constant);
        
        // Implementation would integrate coordinator with the assistant
        todo!("Implement tool integration")
    }
}
```

### Model Management

```rust
// @context_id: ai.ollama.model_management
// @purpose: Automatic model management and optimization
use ollama_rs::models::LocalModel;

impl OpenCircuitOllamaClient {
    pub async fn list_available_models(&self) -> anyhow::Result<Vec<LocalModel>> {
        Ok(self.client.list_local_models().await?)
    }
    
    pub async fn ensure_model_available(&self, model_name: &str) -> anyhow::Result<bool> {
        let models = self.list_available_models().await?;
        let model_exists = models.iter().any(|m| m.name == model_name);
        
        if !model_exists {
            println!("Model {} not found. Please run: ollama pull {}", model_name, model_name);
            return Ok(false);
        }
        
        Ok(true)
    }
    
    pub async fn get_model_info(&self, model_name: &str) -> anyhow::Result<String> {
        let info = self.client.show_model_info(model_name.to_string()).await?;
        Ok(format!("Model: {}\nSize: {} parameters", model_name, info.details.parameter_size))
    }
}
```

## 🚀 Performance Optimization

### Memory Management

```rust
// @context_id: ai.ollama.performance
// @purpose: Optimize memory usage and inference speed
use std::time::Duration;
use tokio::time::timeout;

impl OpenCircuitOllamaClient {
    pub async fn chat_with_timeout(&mut self, message: &str, timeout_secs: u64) -> anyhow::Result<String> {
        let chat_future = self.chat(message);
        
        match timeout(Duration::from_secs(timeout_secs), chat_future).await {
            Ok(result) => result,
            Err(_) => Err(anyhow::anyhow!("Chat request timed out after {} seconds", timeout_secs)),
        }
    }
    
    pub fn clear_history(&mut self) {
        // Keep only system prompt
        if let Some(system_msg) = self.conversation_history.first().cloned() {
            if system_msg.role == "system" {
                self.conversation_history = vec![system_msg];
            } else {
                self.conversation_history.clear();
            }
        }
    }
    
    pub fn trim_history(&mut self, max_messages: usize) {
        if self.conversation_history.len() > max_messages {
            let system_msg = self.conversation_history.first().cloned();
            let recent_messages = self.conversation_history
                .iter()
                .skip(self.conversation_history.len() - max_messages + 1)
                .cloned()
                .collect::<Vec<_>>();
            
            self.conversation_history = if let Some(sys_msg) = system_msg {
                if sys_msg.role == "system" {
                    let mut history = vec![sys_msg];
                    history.extend(recent_messages);
                    history
                } else {
                    recent_messages
                }
            } else {
                recent_messages
            };
        }
    }
}
```

## 🔍 Troubleshooting

### Common Issues

1. **Connection Refused**
   ```bash
   # Check if Ollama is running
   curl http://localhost:11434/api/tags
   
   # Start Ollama if not running
   ollama serve
   ```

2. **Model Not Found**
   ```bash
   # List available models
   ollama list
   
   # Pull required model
   ollama pull qwen2.5:1b
   ```

3. **Out of Memory**
   ```bash
   # Use smaller model
   ollama pull qwen2.5:0.5b
   
   # Or configure memory limits
   export OLLAMA_MAX_LOADED_MODELS=1
   ```

4. **Slow Responses**
   ```bash
   # Enable GPU acceleration (if available)
   # NVIDIA GPU
   export CUDA_VISIBLE_DEVICES=0
   
   # AMD GPU (ROCm)
   export HIP_VISIBLE_DEVICES=0
   ```

### Error Handling

```rust
// @context_id: ai.ollama.error_handling
// @purpose: Robust error handling for production use
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OllamaError {
    #[error("Connection failed: {0}")]
    ConnectionError(String),
    
    #[error("Model not available: {0}")]
    ModelNotFound(String),
    
    #[error("Request timeout after {0} seconds")]
    Timeout(u64),
    
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
    
    #[error("Rate limit exceeded")]
    RateLimit,
}

impl OpenCircuitOllamaClient {
    pub async fn chat_with_retry(&mut self, message: &str, max_retries: u32) -> Result<String, OllamaError> {
        let mut attempts = 0;
        
        while attempts < max_retries {
            match self.chat(message).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    attempts += 1;
                    if attempts >= max_retries {
                        return Err(OllamaError::ConnectionError(e.to_string()));
                    }
                    
                    // Exponential backoff
                    let delay = Duration::from_millis(100 * 2_u64.pow(attempts));
                    tokio::time::sleep(delay).await;
                }
            }
        }
        
        Err(OllamaError::ConnectionError("Max retries exceeded".to_string()))
    }
}
```

## 📚 Additional Resources

- **Ollama Documentation**: https://github.com/ollama/ollama
- **ollama-rs Crate**: https://github.com/pepperoni21/ollama-rs
- **Model Library**: https://ollama.ai/library
- **Qwen Models**: https://huggingface.co/Qwen
- **Performance Tuning**: https://github.com/ollama/ollama/blob/main/docs/faq.md

---

*Last Updated: 2025-01-27*  
*Version: 1.0*  
*Status: Production Ready*