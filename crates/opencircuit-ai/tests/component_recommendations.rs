//! Integration tests for AI-powered component recommendations
//! 
//! These tests verify that the component recommendation system works correctly
//! with real component data and AI models.

use opencircuit_ai::{
    component_advisor::{ComponentAdvisor, RecommendationRequest, PerformancePriority, BudgetConstraints, CostPriority},
    embeddings::ComponentEmbeddingEngine,
    ollama_client::OpenCircuitOllamaClient,
};
use opencircuit_core::models::{Component, ComponentCategory, SpecValue};
use std::collections::HashMap;

/// Create test components for recommendation testing
fn create_test_components() -> Vec<Component> {
    let mut components = Vec::new();

    // Resistors
    let mut resistor_specs = HashMap::new();
    resistor_specs.insert("Resistance".to_string(), SpecValue::String("10k".to_string()));
    resistor_specs.insert("Power".to_string(), SpecValue::String("0.25W".to_string()));
    resistor_specs.insert("Tolerance".to_string(), SpecValue::String("5%".to_string()));
    resistor_specs.insert("Package".to_string(), SpecValue::String("0805".to_string()));

    components.push(
        Component::new(
            "R1001".to_string(),
            "Vishay".to_string(),
            ComponentCategory::Resistors,
            "10k ohm precision resistor".to_string(),
        ).with_specifications(resistor_specs.clone())
    );

    // Capacitors
    let mut capacitor_specs = HashMap::new();
    capacitor_specs.insert("Capacitance".to_string(), SpecValue::String("100nF".to_string()));
    capacitor_specs.insert("Voltage".to_string(), SpecValue::String("50V".to_string()));
    capacitor_specs.insert("Type".to_string(), SpecValue::String("Ceramic".to_string()));
    capacitor_specs.insert("Package".to_string(), SpecValue::String("0805".to_string()));

    components.push(
        Component::new(
            "C2001".to_string(),
            "Murata".to_string(),
            ComponentCategory::Capacitors,
            "100nF ceramic capacitor".to_string(),
        ).with_specifications(capacitor_specs)
    );

    // Transistors
    let mut transistor_specs = HashMap::new();
    transistor_specs.insert("Type".to_string(), SpecValue::String("NPN".to_string()));
    transistor_specs.insert("Voltage".to_string(), SpecValue::String("40V".to_string()));
    transistor_specs.insert("Current".to_string(), SpecValue::String("200mA".to_string()));
    transistor_specs.insert("Package".to_string(), SpecValue::String("SOT-23".to_string()));

    components.push(
        Component::new(
            "Q3001".to_string(),
            "ON Semiconductor".to_string(),
            ComponentCategory::Transistors,
            "NPN general purpose transistor".to_string(),
        ).with_specifications(transistor_specs)
    );

    // Add more resistors with different values
    let mut resistor_1k_specs = HashMap::new();
    resistor_1k_specs.insert("Resistance".to_string(), SpecValue::String("1k".to_string()));
    resistor_1k_specs.insert("Power".to_string(), SpecValue::String("0.125W".to_string()));
    resistor_1k_specs.insert("Tolerance".to_string(), SpecValue::String("1%".to_string()));
    resistor_1k_specs.insert("Package".to_string(), SpecValue::String("0603".to_string()));

    components.push(
        Component::new(
            "R1002".to_string(),
            "Yageo".to_string(),
            ComponentCategory::Resistors,
            "1k ohm precision resistor".to_string(),
        ).with_specifications(resistor_1k_specs)
    );

    components
}

#[tokio::test]
async fn test_component_recommendation_basic() {
    let ollama_client = OpenCircuitOllamaClient::new();
    let mut advisor = ComponentAdvisor::new(ollama_client).await.unwrap();
    
    // Load test components
    let components = create_test_components();
    advisor.load_components(components);

    // Test basic recommendation request
    let request = RecommendationRequest {
        requirements: "I need a pull-up resistor for a 3.3V digital signal".to_string(),
        circuit_context: None,
        preferred_categories: vec![ComponentCategory::Resistors],
        budget_constraints: None,
        performance_priorities: vec![PerformancePriority::Reliability],
        max_recommendations: 3,
    };

    let recommendations = advisor.get_recommendations(request).await.unwrap();
    
    // Should get at least one recommendation
    assert!(!recommendations.is_empty());
    
    // All recommendations should be resistors
    for rec in &recommendations {
        assert_eq!(rec.component.category, ComponentCategory::Resistors);
    }
    
    // Should have confidence scores
    for rec in &recommendations {
        assert!(rec.confidence >= 0.0 && rec.confidence <= 1.0);
    }
}

#[tokio::test]
async fn test_component_recommendation_with_budget() {
    let ollama_client = OpenCircuitOllamaClient::new();
    let mut advisor = ComponentAdvisor::new(ollama_client).await.unwrap();
    
    let components = create_test_components();
    advisor.load_components(components);

    let budget = BudgetConstraints {
        max_cost_per_component: 1.0,
        total_budget: Some(10.0),
        currency: "USD".to_string(),
        cost_priority: CostPriority::MinimizeCost,
    };

    let request = RecommendationRequest {
        requirements: "Low cost capacitor for decoupling".to_string(),
        circuit_context: None,
        preferred_categories: vec![ComponentCategory::Capacitors],
        budget_constraints: Some(budget),
        performance_priorities: vec![PerformancePriority::Size],
        max_recommendations: 2,
    };

    let recommendations = advisor.get_recommendations(request).await.unwrap();
    
    assert!(!recommendations.is_empty());
    
    // Check that cost analysis is present when budget constraints are specified
    for rec in &recommendations {
        if let Some(cost_analysis) = &rec.cost_analysis {
            assert_eq!(cost_analysis.currency, "USD");
            assert!(cost_analysis.unit_cost <= 1.0);
        }
    }
}

#[tokio::test]
async fn test_category_specific_recommendations() {
    let ollama_client = OpenCircuitOllamaClient::new();
    let mut advisor = ComponentAdvisor::new(ollama_client).await.unwrap();
    
    let components = create_test_components();
    advisor.load_components(components);

    // Test transistor recommendations
    let recommendations = advisor.get_category_recommendations(
        ComponentCategory::Transistors,
        "Need a switching transistor for LED driver",
        5
    ).await.unwrap();

    assert!(!recommendations.is_empty());
    
    for rec in &recommendations {
        assert_eq!(rec.component.category, ComponentCategory::Transistors);
        assert!(!rec.reasoning.is_empty());
    }
}

#[tokio::test]
async fn test_component_alternatives() {
    let ollama_client = OpenCircuitOllamaClient::new();
    let mut advisor = ComponentAdvisor::new(ollama_client).await.unwrap();
    
    let components = create_test_components();
    advisor.load_components(components.clone());

    // Get alternatives for the first resistor
    let reference_component = &components[0]; // 10k resistor
    
    let alternatives = advisor.get_alternatives(
        reference_component,
        "Need similar resistor with better tolerance",
        3
    ).await.unwrap();

    // Should find some alternatives
    assert!(!alternatives.is_empty());
    
    // Alternatives should be in the same category
    for alt in &alternatives {
        assert_eq!(alt.component.category, reference_component.category);
    }
}

#[tokio::test]
async fn test_embedding_similarity_search() {
    let ollama_client = OpenCircuitOllamaClient::new();
    let mut engine = ComponentEmbeddingEngine::new(ollama_client).await.unwrap();
    
    let components = create_test_components();
    
    // Test similarity search by requirements
    let matches = engine.find_similar_components_by_requirements(
        "10k ohm resistor for voltage divider",
        &components,
        3
    ).await.unwrap();

    assert!(!matches.is_empty());
    
    // Should find the 10k resistor with high similarity
    let best_match = &matches[0];
    assert!(best_match.similarity > 0.3);
    assert!(!best_match.match_reason.is_empty());
}

#[tokio::test]
async fn test_performance_priorities() {
    let ollama_client = OpenCircuitOllamaClient::new();
    let mut advisor = ComponentAdvisor::new(ollama_client).await.unwrap();
    
    let components = create_test_components();
    advisor.load_components(components);

    // Test with different performance priorities
    let priorities = vec![
        PerformancePriority::PowerEfficiency,
        PerformancePriority::Size,
        PerformancePriority::Reliability,
    ];

    let request = RecommendationRequest {
        requirements: "Capacitor for power supply filtering".to_string(),
        circuit_context: None,
        preferred_categories: vec![ComponentCategory::Capacitors],
        budget_constraints: None,
        performance_priorities: priorities,
        max_recommendations: 2,
    };

    let recommendations = advisor.get_recommendations(request).await.unwrap();
    
    assert!(!recommendations.is_empty());
    
    // Should include performance notes when priorities are specified
    for rec in &recommendations {
        assert!(!rec.performance_notes.is_empty());
    }
}

#[tokio::test]
async fn test_embedding_cache_functionality() {
    let ollama_client = OpenCircuitOllamaClient::new();
    let mut engine = ComponentEmbeddingEngine::new(ollama_client).await.unwrap();
    
    let components = create_test_components();
    let component = &components[0];
    
    // Generate embedding first time
    let embedding1 = engine.generate_component_embedding(component).await.unwrap();
    
    // Generate embedding second time (should use cache)
    let embedding2 = engine.generate_component_embedding(component).await.unwrap();
    
    // Should be identical (from cache)
    assert_eq!(embedding1.component_id, embedding2.component_id);
    assert_eq!(embedding1.vector, embedding2.vector);
    
    // Check cache stats
    let (count, _memory) = engine.cache_stats();
    assert!(count > 0);
    
    // Clear cache and verify
    engine.clear_cache();
    let (count_after_clear, _) = engine.cache_stats();
    assert_eq!(count_after_clear, 0);
}