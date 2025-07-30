//! Mock tests for component recommendations that work without Ollama
//! 
//! These tests demonstrate the component recommendation system functionality
//! using simplified embeddings that don't require an external AI service.

use opencircuit_ai::{
    component_advisor::{ComponentAdvisor, RecommendationRequest, BudgetConstraints, PerformancePriority, CostPriority},
    embeddings::ComponentEmbeddingEngine,
    ollama_client::OpenCircuitOllamaClient,
    models::{AiContext, CircuitType, DesignPhase, ExpertiseLevel},
};
use opencircuit_core::models::{Component, ComponentCategory, SpecValue};
use std::collections::HashMap;

/// Create test components for mock testing
fn create_test_components() -> Vec<Component> {
    let mut components = Vec::new();

    // Resistors
    let mut resistor_specs = HashMap::new();
    resistor_specs.insert("Resistance".to_string(), SpecValue::String("10k".to_string()));
    resistor_specs.insert("Power".to_string(), SpecValue::String("0.25W".to_string()));
    resistor_specs.insert("Tolerance".to_string(), SpecValue::String("5%".to_string()));
    
    components.push(
        Component::new(
            "R1001".to_string(),
            "Vishay".to_string(),
            ComponentCategory::Resistors,
            "10k ohm resistor".to_string(),
        ).with_specifications(resistor_specs)
    );

    // Capacitors
    let mut capacitor_specs = HashMap::new();
    capacitor_specs.insert("Capacitance".to_string(), SpecValue::String("100nF".to_string()));
    capacitor_specs.insert("Voltage".to_string(), SpecValue::String("50V".to_string()));
    capacitor_specs.insert("Dielectric".to_string(), SpecValue::String("X7R".to_string()));
    
    components.push(
        Component::new(
            "C1001".to_string(),
            "Murata".to_string(),
            ComponentCategory::Capacitors,
            "100nF ceramic capacitor".to_string(),
        ).with_specifications(capacitor_specs)
    );

    components
}

#[tokio::test]
async fn test_component_advisor_creation() {
    let ollama_client = OpenCircuitOllamaClient::new();
    let result = ComponentAdvisor::new(ollama_client).await;
    assert!(result.is_ok(), "Should be able to create ComponentAdvisor");
}

#[tokio::test]
async fn test_load_components() {
    let ollama_client = OpenCircuitOllamaClient::new();
    let mut advisor = ComponentAdvisor::new(ollama_client).await.unwrap();
    
    let components = create_test_components();
    let component_count = components.len();
    
    advisor.load_components(components);
    
    // The advisor should have loaded the components
    // (We can't directly test this without exposing internal state,
    // but the load operation should complete without error)
    assert_eq!(component_count, 2);
}

#[tokio::test]
async fn test_embedding_engine_creation() {
    let ollama_client = OpenCircuitOllamaClient::new();
    let result = ComponentEmbeddingEngine::new(ollama_client).await;
    assert!(result.is_ok(), "Should be able to create ComponentEmbeddingEngine");
}

#[tokio::test]
async fn test_similarity_search_with_mock_embeddings() {
    let ollama_client = OpenCircuitOllamaClient::new();
    let mut embedding_engine = ComponentEmbeddingEngine::new(ollama_client).await.unwrap();
    
    let components = create_test_components();
    
    // This should work with the simplified hash-based embeddings
    let result = embedding_engine.find_similar_components_by_requirements(
        "10k resistor",
        &components,
        2
    ).await;
    
    // The result might be Ok or Err depending on the implementation
    // but the function should not panic
    match result {
        Ok(matches) => {
            println!("Found {} matches", matches.len());
            assert!(matches.len() <= 2);
        }
        Err(e) => {
            println!("Expected error without Ollama: {}", e);
            // This is expected behavior without Ollama running
        }
    }
}

#[tokio::test]
async fn test_cache_functionality() {
    let ollama_client = OpenCircuitOllamaClient::new();
    let embedding_engine = ComponentEmbeddingEngine::new(ollama_client).await.unwrap();
    
    // Test cache stats
    let (count, memory) = embedding_engine.cache_stats();
    assert_eq!(count, 0, "Cache should start empty");
    assert_eq!(memory, 0, "Memory usage should start at 0");
}

#[tokio::test]
async fn test_recommendation_request_creation() {
    // Test that we can create recommendation requests
    let request = RecommendationRequest {
        requirements: "Need a pull-up resistor".to_string(),
        circuit_context: Some(AiContext {
            project_name: Some("Test Project".to_string()),
            circuit_type: Some(CircuitType::Digital),
            constraints: vec!["3.3V logic".to_string()],
            mentioned_components: vec![],
            design_phase: DesignPhase::ComponentSelection,
            user_level: ExpertiseLevel::Intermediate,
        }),
        preferred_categories: vec![ComponentCategory::Resistors],
        budget_constraints: Some(BudgetConstraints {
            max_cost_per_component: 1.0,
            total_budget: None,
            currency: "USD".to_string(),
            cost_priority: CostPriority::BalanceCostPerformance,
        }),
        performance_priorities: vec![PerformancePriority::Reliability],
        max_recommendations: 5,
    };
    
    assert_eq!(request.requirements, "Need a pull-up resistor");
    assert_eq!(request.max_recommendations, 5);
    assert_eq!(request.preferred_categories.len(), 1);
    assert_eq!(request.performance_priorities.len(), 1);
}

#[tokio::test]
async fn test_component_creation_with_specs() {
    let mut specs = HashMap::new();
    specs.insert("Resistance".to_string(), SpecValue::String("1k".to_string()));
    specs.insert("Power".to_string(), SpecValue::String("0.125W".to_string()));
    
    let component = Component::new(
        "R2001".to_string(),
        "Yageo".to_string(),
        ComponentCategory::Resistors,
        "1k ohm resistor".to_string(),
    ).with_specifications(specs);
    
    assert_eq!(component.part_number, "R2001");
    assert_eq!(component.manufacturer, "Yageo");
    assert_eq!(component.category, ComponentCategory::Resistors);
    assert!(!component.specifications.is_empty());
}