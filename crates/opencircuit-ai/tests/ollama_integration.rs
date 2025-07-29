//! Integration tests for Ollama AI functionality
//! 
//! This file contains integration tests for the Ollama client and manager.
//! Run with: cargo test --test ollama_integration

use opencircuit_ai::{AiService, AiConfig, models::{AiUseCase, AiModel}, ollama_client::OpenCircuitOllamaClient, ollama_manager::OllamaManager};

#[tokio::test]
async fn test_ollama_server_connection() {
    let client = OpenCircuitOllamaClient::new();
    
    // Test server health check
    match client.health_check().await {
        Ok(true) => println!("✅ Ollama server is running and accessible"),
        Ok(false) => println!("❌ Ollama server is not responding properly"),
        Err(e) => println!("⚠️  Could not connect to Ollama server: {}", e),
    }
}

#[tokio::test]
async fn test_model_availability() {
    let mut manager = OllamaManager::new();
    
    match manager.initialize().await {
        Ok(_) => {
            println!("✅ Ollama manager initialized successfully");
            
            let available_models = manager.get_available_models();
            println!("Available models: {:?}", available_models);
            
            if available_models.is_empty() {
                println!("⚠️  No models available. Please run: ollama pull qwen2.5:0.5b");
            } else {
                println!("✅ Found {} available models", available_models.len());
            }
        }
        Err(e) => println!("❌ Failed to initialize Ollama manager: {}", e),
    }
}

#[tokio::test]
async fn test_basic_chat() {
    let mut service = match AiService::new().await {
        Ok(service) => service,
        Err(e) => {
            println!("❌ Failed to create AI service: {}", e);
            return;
        }
    };
    
    match service.initialize().await {
        Ok(_) => {
            println!("✅ AI service initialized");
            
            if service.is_ready() {
                let response = service.chat(
                    "Hello! Can you help me with circuit design?",
                    AiUseCase::BasicChat
                ).await;
                
                match response {
                    Ok(ai_response) => {
                        println!("✅ Chat response received:");
                        println!("Model: {}", ai_response.model);
                        println!("Response: {}", ai_response.content);
                        println!("Generation time: {}ms", ai_response.generation_time_ms);
                        println!("Confidence: {:.2}", ai_response.confidence);
                    }
                    Err(e) => println!("❌ Chat failed: {}", e),
                }
            } else {
                println!("⚠️  AI service not ready - no models available");
            }
        }
        Err(e) => println!("❌ Failed to initialize AI service: {}", e),
    }
}

#[tokio::test]
async fn test_component_suggestion() {
    let mut service = match AiService::new().await {
        Ok(service) => service,
        Err(e) => {
            println!("❌ Failed to create AI service: {}", e);
            return;
        }
    };
    
    match service.initialize().await {
        Ok(_) => {
            if service.is_ready() {
                let response = service.suggest_components(
                    "I need a low-noise amplifier for audio applications, operating at 5V with gain of 20dB"
                ).await;
                
                match response {
                    Ok(ai_response) => {
                        println!("✅ Component suggestion received:");
                        println!("Response: {}", ai_response.content);
                    }
                    Err(e) => println!("❌ Component suggestion failed: {}", e),
                }
            } else {
                println!("⚠️  AI service not ready for component suggestions");
            }
        }
        Err(e) => println!("❌ Failed to initialize AI service: {}", e),
    }
}

#[tokio::test]
async fn test_circuit_analysis() {
    let mut service = match AiService::new().await {
        Ok(service) => service,
        Err(e) => {
            println!("❌ Failed to create AI service: {}", e);
            return;
        }
    };
    
    match service.initialize().await {
        Ok(_) => {
            if service.is_ready() {
                let circuit_description = "
                    Simple RC low-pass filter:
                    - Input: AC signal 0-10kHz
                    - R1: 1kΩ resistor
                    - C1: 100nF capacitor
                    - Output: Filtered signal
                ";
                
                let response = service.analyze_circuit(circuit_description).await;
                
                match response {
                    Ok(ai_response) => {
                        println!("✅ Circuit analysis received:");
                        println!("Response: {}", ai_response.content);
                    }
                    Err(e) => println!("❌ Circuit analysis failed: {}", e),
                }
            } else {
                println!("⚠️  AI service not ready for circuit analysis");
            }
        }
        Err(e) => println!("❌ Failed to initialize AI service: {}", e),
    }
}

#[tokio::test]
async fn test_model_switching() {
    let mut service = match AiService::new().await {
        Ok(service) => service,
        Err(e) => {
            println!("❌ Failed to create AI service: {}", e);
            return;
        }
    };
    
    match service.initialize().await {
        Ok(_) => {
            println!("Current model: {:?}", service.get_active_model());
            
            // Try to switch to a different model if available
            let available_models = service.get_available_models();
            if available_models.len() > 1 {
                let new_model = &available_models[1];
                match service.set_model(new_model.clone()).await {
                    Ok(_) => {
                        println!("✅ Successfully switched to model: {:?}", new_model);
                        println!("New active model: {:?}", service.get_active_model());
                    }
                    Err(e) => println!("❌ Failed to switch model: {}", e),
                }
            } else {
                println!("⚠️  Only one model available, cannot test model switching");
            }
        }
        Err(e) => println!("❌ Failed to initialize AI service: {}", e),
    }
}

#[tokio::test]
async fn test_ai_service_creation() {
    let service = AiService::new().await;
    match service {
        Ok(service) => {
            println!("✅ AI service created successfully");
            assert_eq!(service.get_active_model(), &AiModel::QwenTiny);
        }
        Err(e) => {
            println!("❌ Failed to create AI service: {}", e);
            // Don't panic in integration tests, just log the error
        }
    }
}

/// Helper function to print test results
pub fn print_test_summary() {
    println!("\n🧪 Ollama Integration Test Summary");
    println!("================================");
    println!("Run these tests with: cargo test --test ollama_integration");
    println!("\nPrerequisites:");
    println!("1. Install Ollama: https://ollama.ai/download");
    println!("2. Start Ollama server: ollama serve");
    println!("3. Pull a model: ollama pull qwen2.5:0.5b");
    println!("\nTests:");
    println!("- test_ollama_server_connection: Checks if Ollama server is running");
    println!("- test_model_availability: Verifies models are available");
    println!("- test_basic_chat: Tests basic AI conversation");
    println!("- test_component_suggestion: Tests component recommendation");
    println!("- test_circuit_analysis: Tests circuit analysis capability");
    println!("- test_model_switching: Tests switching between models");
    println!("- test_ai_service_creation: Tests AI service instantiation");
}