---
title: AI & Machine Learning Integration
description: AI-powered features and machine learning integration in OpenCircuit using Ollama
last_updated: 2025-01-27
tags: [ai, ml, machine-learning, ollama, local-models, llm, assistant]
context_id: ai.overview.main
---

# 🤖 AI & Machine Learning Integration

OpenCircuit leverages advanced AI capabilities to enhance circuit design, analysis, and user interaction. This document provides a comprehensive overview of the AI integration architecture and features.

## AI Integration Overview

OpenCircuit leverages local AI models through Ollama to provide intelligent design assistance, automated optimization, and enhanced user experience while maintaining privacy and offline capabilities.

### Core AI Engine

OpenCircuit uses **Ollama** as its primary AI engine, providing local, privacy-focused AI processing without requiring external API keys or internet connectivity for core functionality.

#### Key Components:
- **Ollama Server**: Local AI inference server running on localhost:11434
- **Model Management**: Automatic downloading and management of AI models
- **Rust Integration**: Native integration using `ollama-rs` crate
- **Streaming Responses**: Real-time AI responses for better user experience
- **Progressive Loading**: Start with ultra-lightweight models and scale up

### Model Strategy

OpenCircuit employs a progressive testing and scaling strategy optimized for different use cases:

#### Progressive Model Testing:
1. **qwen2.5:0.5b** - Ultra-lightweight for initial validation and testing
2. **qwen2.5:1b** - Balanced performance for production use
3. **qwen2.5:3b** - Advanced reasoning for complex circuit analysis

#### Model Selection Criteria:
- **Performance**: Response time and accuracy validation
- **Resource Usage**: Memory and CPU requirements testing
- **Specialization**: Circuit design and electronics knowledge
- **Local Processing**: No external dependencies
- **Scalability**: Progressive enhancement based on hardware capabilities

### Benefits of Local AI Processing

- **Privacy**: All processing happens locally, no data leaves your system
- **Performance**: No network latency, instant responses
- **Cost**: No API usage fees or subscription costs
- **Reliability**: Works completely offline
- **Control**: Full control over model behavior and updates
- **Testing**: Start small and scale up based on actual performance

## 🧠 AI Architecture Overview

```
┌─────────────────────────────────────┐
│         OpenCircuit Frontend       │
├─────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐   │
│  │ AI Assistant│  │ Chat Panel  │   │
│  │ (Ollama)    │  │   (egui)    │   │
│  └─────────────┘  └─────────────┘   │
├─────────────────────────────────────┤
│         Vector Database            │
│      (Component Knowledge)         │
├─────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐   │
│  │   Ollama    │  │ Local Models│   │
│  │ (Local API) │  │ (Qwen/Gemma)│   │
│  └─────────────┘  └─────────────┘   │
└─────────────────────────────────────┘
```

## 🦀 Rust ML Ecosystem

### Candle Framework
**URL:** `https://github.com/huggingface/candle`
**Context ID:** `ai.candle.framework`

Minimalist ML framework for Rust with GPU support:

```rust
// @context_id: ai.candle.basic_usage
// @purpose: Basic tensor operations with Candle
use candle_core::{Device, Tensor, Result};
use candle_nn::{linear, Linear, Module, VarBuilder};

pub struct ComponentClassifier {
    linear1: Linear,
    linear2: Linear,
    device: Device,
}

impl ComponentClassifier {
    pub fn new(vs: VarBuilder, input_dim: usize, hidden_dim: usize, output_dim: usize) -> Result<Self> {
        let linear1 = linear(input_dim, hidden_dim, vs.pp("linear1"))?;
        let linear2 = linear(hidden_dim, output_dim, vs.pp("linear2"))?;
        let device = vs.device().clone();
        
        Ok(Self { linear1, linear2, device })
    }
    
    pub fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let x = self.linear1.forward(input)?;
        let x = x.relu()?;
        let x = self.linear2.forward(&x)?;
        Ok(x)
    }
    
    pub fn classify_component(&self, features: Vec<f32>) -> Result<ComponentType> {
        let input = Tensor::from_vec(features, (1, features.len()), &self.device)?;
        let output = self.forward(&input)?;
        let probabilities = output.softmax(1)?;
        
        // Get the class with highest probability
        let class_idx = probabilities.argmax(1)?.to_scalar::<u32>()?;
        Ok(ComponentType::from_index(class_idx as usize))
    }
}
```

### Additional ML Crates
- **tch** - PyTorch bindings for Rust
- **ort** - ONNX Runtime integration
- **linfa** - Comprehensive ML toolkit

```rust
// @context_id: ai.ml.crates_integration
// @purpose: Integration with multiple ML frameworks
use tch::{nn, Device as TchDevice, Tensor as TchTensor};
use ort::{Environment, SessionBuilder, Value};

pub struct HybridMLEngine {
    candle_model: ComponentClassifier,
    pytorch_model: nn::Sequential,
    onnx_session: ort::Session,
}

impl HybridMLEngine {
    pub async fn predict_component_placement(&self, circuit_data: &CircuitData) -> Result<PlacementSuggestion> {
        // Use Candle for component classification
        let component_types = self.classify_components_candle(circuit_data).await?;
        
        // Use PyTorch for placement optimization
        let placement_scores = self.optimize_placement_pytorch(&component_types).await?;
        
        // Use ONNX for final refinement
        let refined_placement = self.refine_placement_onnx(&placement_scores).await?;
        
        Ok(PlacementSuggestion {
            components: component_types,
            positions: refined_placement,
            confidence: self.calculate_confidence(&placement_scores),
        })
    }
}
```

## 🦙 Ollama Integration

### Local AI Model Setup
**URL:** `https://ollama.ai`
**Context ID:** `ai.ollama.integration`

Ollama provides local AI model inference with privacy and offline capabilities:

```rust
// @context_id: ai.ollama.basic_usage
// @purpose: Basic Ollama integration with OpenCircuit
use ollama_rs::{Ollama, generation::completion::GenerationRequest};
use ollama_rs::generation::chat::{ChatMessage, ChatMessageRequest};

pub struct CircuitAssistant {
    client: Ollama,
    model: String,
    context_history: Vec<ChatMessage>,
}

impl CircuitAssistant {
    pub fn new(model: String) -> Self {
        let client = Ollama::default(); // localhost:11434
        let system_prompt = "You are an expert electronics engineer specializing in circuit design, component selection, and PCB layout. Provide practical, implementable advice with specific component values and part numbers.";
        
        let mut context_history = vec![
            ChatMessage::system(system_prompt.to_string())
        ];
        
        Self { client, model, context_history }
    }
    
    pub async fn ask_circuit_question(&mut self, question: &str, circuit_context: &str) -> Result<String> {
        let enhanced_question = format!(
            "Circuit Context: {}\n\nQuestion: {}",
            circuit_context, question
        );
        
        self.context_history.push(ChatMessage::user(enhanced_question.clone()));
        
        let response = self.client
            .send_chat_messages_with_history(
                &mut self.context_history,
                ChatMessageRequest::new(
                    self.model.clone(),
                    vec![ChatMessage::user(enhanced_question)],
                ),
            )
            .await?;
        
        Ok(response.message.content)
    }
    
    pub async fn suggest_components(&mut self, requirements: &str) -> Result<ComponentSuggestion> {
        let prompt = format!(
            "Based on these requirements, suggest specific electronic components with part numbers and specifications:\n\n{}",
            requirements
        );
        
        let response = self.ask_circuit_question(&prompt, "component_selection").await?;
        
        Ok(ComponentSuggestion {
            components: self.parse_component_suggestions(&response),
            reasoning: response,
            confidence: self.calculate_confidence(&response),
        })
    }
}
```

### Recommended Models for Circuit Design

```rust
// @context_id: ai.ollama.model_selection
// @purpose: Model recommendations for different use cases
pub enum CircuitDesignModel {
    // Ultra-lightweight for testing and quick responses
    QwenMini,      // qwen2.5:0.5b - 400MB, very fast
    
    // Balanced performance for production use
    QwenBalanced,  // qwen2.5:1b - 800MB, good quality/speed
    
    // High-quality analysis for complex circuits
    QwenAdvanced,  // qwen2.5:3b - 2GB, excellent reasoning
    
    // Specialized for code generation (netlists, PCB scripts)
    QwenCoder,     // qwen2.5-coder:1.5b - optimized for code
    
    // Alternative with multimodal capabilities
    Gemma,         // gemma2:2b - Google's model
}

impl CircuitDesignModel {
    pub fn model_name(&self) -> &'static str {
        match self {
            Self::QwenMini => "qwen2.5:0.5b",
            Self::QwenBalanced => "qwen2.5:1b",
            Self::QwenAdvanced => "qwen2.5:3b",
            Self::QwenCoder => "qwen2.5-coder:1.5b",
            Self::Gemma => "gemma2:2b",
        }
    }
    
    pub fn use_case(&self) -> &'static str {
        match self {
            Self::QwenMini => "Quick component lookups, basic questions",
            Self::QwenBalanced => "General circuit design assistance",
            Self::QwenAdvanced => "Complex analysis, optimization",
            Self::QwenCoder => "Netlist generation, PCB scripting",
            Self::Gemma => "Alternative choice, future multimodal",
        }
    }
}
```
}

### Ollama Configuration and Setup
```rust
// @context_id: ai.ollama.configuration
// @purpose: Ollama server configuration and model management
use std::process::Command;
use tokio::time::{sleep, Duration};

pub struct OllamaManager {
    base_url: String,
    models: Vec<String>,
}

impl OllamaManager {
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            models: vec![
                "qwen2.5:0.5b".to_string(),   // Ultra-lightweight for testing
                "qwen2.5:1b".to_string(),     // Balanced performance
                "qwen2.5:3b".to_string(),     // High-quality analysis
                "gemma2:2b".to_string(),      // Alternative model
            ],
        }
    }
    
    pub async fn ensure_model_available(&self, model: &str) -> Result<()> {
        let client = Ollama::new(self.base_url.clone(), 11434);
        
        // Check if model is already available
        let models = client.list_local_models().await?;
        if models.iter().any(|m| m.name == model) {
            return Ok(());
        }
        
        // Pull model if not available
        println!("Downloading model: {}", model);
        client.pull_model(model.to_string(), false).await?;
        
        // Wait for model to be ready
        self.wait_for_model_ready(&client, model).await?;
        
        Ok(())
    }
    
    async fn wait_for_model_ready(&self, client: &Ollama, model: &str) -> Result<()> {
        for _ in 0..30 { // Wait up to 30 seconds
            if let Ok(models) = client.list_local_models().await {
                if models.iter().any(|m| m.name == model) {
                    return Ok(());
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
        Err("Model failed to become ready".into())
    }
    
    pub async fn get_model_info(&self, model: &str) -> Result<ModelInfo> {
        let client = Ollama::new(self.base_url.clone(), 11434);
        let info = client.show_model_info(model.to_string()).await?;
        
        Ok(ModelInfo {
            name: model.to_string(),
            size: info.size,
            parameters: info.details.parameter_size,
            quantization: info.details.quantization_level,
            family: info.details.family,
        })
    }
}

#[derive(Debug)]
pub struct ModelInfo {
    pub name: String,
    pub size: u64,
    pub parameters: String,
    pub quantization: String,
    pub family: String,
}
```

### Streaming Responses for Real-time Feedback
```rust
// @context_id: ai.ollama.streaming
// @purpose: Real-time streaming responses for better UX
use futures::StreamExt;
use ollama_rs::generation::completion::GenerationRequest;

impl CircuitAssistant {
    pub async fn stream_circuit_analysis<F>(&mut self, 
        circuit_data: &str, 
        mut callback: F
    ) -> Result<String>
    where
        F: FnMut(&str) + Send,
    {
        let prompt = format!(
            "Analyze this circuit step by step, explaining each component and connection:\n\n{}",
            circuit_data
        );
        
        let request = GenerationRequest::new(
            self.model.clone(),
            prompt,
        ).stream();
        
        let mut stream = self.client.generate_stream(request).await?;
        let mut full_response = String::new();
        
        while let Some(response) = stream.next().await {
            match response {
                Ok(generation) => {
                    if let Some(response_text) = generation.response {
                        full_response.push_str(&response_text);
                        callback(&response_text); // Real-time callback
                    }
                    
                    if generation.done {
                        break;
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
        
        Ok(full_response)
    }
}
```

## 🗄️ Vector Databases

### Qdrant Integration
```rust
// @context_id: ai.vector.qdrant
// @purpose: Component knowledge base with Qdrant
use qdrant_client::{client::QdrantClient, qdrant::{CreateCollection, VectorParams, Distance}};

pub struct ComponentKnowledgeBase {
    client: QdrantClient,
    collection_name: String,
}

impl ComponentKnowledgeBase {
    pub async fn new() -> Result<Self> {
        let client = QdrantClient::from_url("http://localhost:6334").build()?;
        let collection_name = "component_knowledge".to_string();
        
        // Create collection for component embeddings
        client.create_collection(&CreateCollection {
            collection_name: collection_name.clone(),
            vectors_config: Some(VectorParams {
                size: 384, // Sentence transformer dimension
                distance: Distance::Cosine as i32,
                ..Default::default()
            }.into()),
            ..Default::default()
        }).await?;
        
        Ok(Self { client, collection_name })
    }
    
    pub async fn search_similar_components(&self, query_embedding: Vec<f32>, limit: u64) -> Result<Vec<ComponentInfo>> {
        let search_result = self.client
            .search_points(&SearchPoints {
                collection_name: self.collection_name.clone(),
                vector: query_embedding,
                limit,
                with_payload: Some(true.into()),
                ..Default::default()
            })
            .await?;
        
        // Convert search results to ComponentInfo
        let components = search_result.result
            .into_iter()
            .map(|point| ComponentInfo::from_payload(point.payload))
            .collect::<Result<Vec<_>>>()?;
        
        Ok(components)
    }
}
```

## 🎯 AI-Powered Features

### Intelligent Component Placement
```rust
// @context_id: ai.features.placement
// @purpose: AI-driven component placement optimization
pub struct IntelligentPlacer {
    ml_engine: HybridMLEngine,
    knowledge_base: ComponentKnowledgeBase,
}

impl IntelligentPlacer {
    pub async fn suggest_placement(&self, circuit: &Circuit) -> Result<PlacementSuggestion> {
        // Extract circuit features
        let features = self.extract_circuit_features(circuit)?;
        
        // Get similar designs from knowledge base
        let similar_designs = self.knowledge_base
            .search_similar_components(features.embedding, 10)
            .await?;
        
        // Use ML model to predict optimal placement
        let placement = self.ml_engine
            .predict_component_placement(&features)
            .await?;
        
        // Combine with knowledge base insights
        self.refine_with_knowledge(placement, similar_designs)
    }
}
```

### Automated Design Rule Checking
```rust
// @context_id: ai.features.drc
// @purpose: AI-enhanced design rule checking
pub struct AIDesignRuleChecker {
    rule_classifier: ComponentClassifier,
    violation_detector: ViolationDetector,
}

impl AIDesignRuleChecker {
    pub async fn check_design(&self, pcb: &PCBDesign) -> Result<Vec<DesignViolation>> {
        let mut violations = Vec::new();
        
        // Traditional rule checking
        violations.extend(self.check_traditional_rules(pcb)?);
        
        // AI-enhanced checking for complex patterns
        let ai_violations = self.detect_ai_violations(pcb).await?;
        violations.extend(ai_violations);
        
        // Prioritize violations by severity
        violations.sort_by_key(|v| v.severity);
        
        Ok(violations)
    }
    
    async fn detect_ai_violations(&self, pcb: &PCBDesign) -> Result<Vec<DesignViolation>> {
        // Extract layout patterns
        let patterns = self.extract_layout_patterns(pcb)?;
        
        // Classify potential issues
        let classifications = self.rule_classifier
            .classify_patterns(&patterns)
            .await?;
        
        // Convert to violations
        self.patterns_to_violations(patterns, classifications)
    }
}
```

## 🔗 Quick Links

- [Rust ML Ecosystem](rust_ml.md)
- [AI Model Integration](integration.md)
- [Vector Databases](databases.md)
- [Performance Optimization](performance.md)

---

*Context ID: ai.overview.main*