//! Demonstration of AI-powered component recommendations
//! 
//! This example shows how to use the component recommendation system
//! without requiring a running Ollama server.

use opencircuit_ai::{
    component_advisor::{ComponentAdvisor, RecommendationRequest, PerformancePriority},
    embeddings::ComponentEmbeddingEngine,
    ollama_client::OpenCircuitOllamaClient,
};
use opencircuit_core::models::{Component, ComponentCategory, SpecValue};
use std::collections::HashMap;

/// Create sample components for demonstration
fn create_sample_components() -> Vec<Component> {
    let mut components = Vec::new();

    // Create various resistors
    for (value, power, tolerance, package) in [
        ("1k", "0.125W", "1%", "0603"),
        ("10k", "0.25W", "5%", "0805"),
        ("100k", "0.125W", "1%", "0603"),
        ("4.7k", "0.25W", "5%", "0805"),
    ] {
        let mut specs = HashMap::new();
        specs.insert("Resistance".to_string(), SpecValue::String(value.to_string()));
        specs.insert("Power".to_string(), SpecValue::String(power.to_string()));
        specs.insert("Tolerance".to_string(), SpecValue::String(tolerance.to_string()));
        specs.insert("Package".to_string(), SpecValue::String(package.to_string()));

        components.push(
            Component::new(
                format!("R{}", components.len() + 1000),
                "Vishay".to_string(),
                ComponentCategory::Resistors,
                format!("{} ohm resistor", value),
            ).with_specifications(specs)
        );
    }

    // Create various capacitors
    for (value, voltage, dielectric, package) in [
        ("100nF", "50V", "X7R", "0805"),
        ("10uF", "25V", "X5R", "1206"),
        ("1uF", "50V", "X7R", "0805"),
        ("22pF", "50V", "C0G", "0603"),
    ] {
        let mut specs = HashMap::new();
        specs.insert("Capacitance".to_string(), SpecValue::String(value.to_string()));
        specs.insert("Voltage".to_string(), SpecValue::String(voltage.to_string()));
        specs.insert("Dielectric".to_string(), SpecValue::String(dielectric.to_string()));
        specs.insert("Package".to_string(), SpecValue::String(package.to_string()));

        components.push(
            Component::new(
                format!("C{}", components.len() - 3),
                "Murata".to_string(),
                ComponentCategory::Capacitors,
                format!("{} ceramic capacitor", value),
            ).with_specifications(specs)
        );
    }

    // Create some transistors
    for (type_name, voltage, current, package) in [
        ("NPN", "40V", "200mA", "SOT-23"),
        ("PNP", "40V", "200mA", "SOT-23"),
        ("N-MOSFET", "60V", "2A", "SOT-23"),
        ("P-MOSFET", "60V", "2A", "SOT-23"),
    ] {
        let mut specs = HashMap::new();
        specs.insert("Type".to_string(), SpecValue::String(type_name.to_string()));
        specs.insert("Voltage".to_string(), SpecValue::String(voltage.to_string()));
        specs.insert("Current".to_string(), SpecValue::String(current.to_string()));
        specs.insert("Package".to_string(), SpecValue::String(package.to_string()));

        components.push(
            Component::new(
                format!("Q{}", components.len() - 7),
                "ON Semiconductor".to_string(),
                ComponentCategory::Transistors,
                format!("{} transistor", type_name),
            ).with_specifications(specs)
        );
    }

    components
}

/// Demonstrate component recommendation functionality
pub async fn demonstrate_component_recommendations() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 AI-Powered Component Recommendations Demo");
    println!("============================================\n");

    // Create the component advisor (this will work even without Ollama running)
    let ollama_client = OpenCircuitOllamaClient::new();
    let mut advisor = ComponentAdvisor::new(ollama_client).await?;
    
    // Load sample components
    let components = create_sample_components();
    println!("📦 Loaded {} sample components:", components.len());
    for component in &components {
        println!("  - {} {} ({})", 
            component.manufacturer, 
            component.part_number, 
            component.description
        );
    }
    println!();

    advisor.load_components(components);

    // Demonstrate different types of recommendations
    println!("🔍 Demonstration Scenarios:");
    println!("---------------------------\n");

    // Scenario 1: Basic resistor recommendation
    println!("1. 📊 Pull-up Resistor Recommendation");
    let request = RecommendationRequest {
        requirements: "I need a pull-up resistor for a 3.3V digital signal".to_string(),
        circuit_context: None,
        preferred_categories: vec![ComponentCategory::Resistors],
        budget_constraints: None,
        performance_priorities: vec![PerformancePriority::Reliability],
        max_recommendations: 3,
    };

    match advisor.get_recommendations(request).await {
        Ok(recommendations) => {
            println!("   Found {} recommendations:", recommendations.len());
            for (i, rec) in recommendations.iter().enumerate() {
                println!("   {}. {} {} - Confidence: {:.1}%", 
                    i + 1,
                    rec.component.manufacturer,
                    rec.component.part_number,
                    rec.confidence * 100.0
                );
                println!("      Reasoning: {}", rec.reasoning);
                if !rec.performance_notes.is_empty() {
                    println!("      Performance: {}", rec.performance_notes.join(", "));
                }
                println!();
            }
        }
        Err(e) => println!("   ⚠️  Recommendation failed (expected without Ollama): {}", e),
    }

    // Scenario 2: Capacitor for power supply
    println!("2. ⚡ Power Supply Capacitor Recommendation");
    match advisor.get_category_recommendations(
        ComponentCategory::Capacitors,
        "Need a decoupling capacitor for a microcontroller power supply",
        2
    ).await {
        Ok(recommendations) => {
            println!("   Found {} recommendations:", recommendations.len());
            for (i, rec) in recommendations.iter().enumerate() {
                println!("   {}. {} {} - Confidence: {:.1}%", 
                    i + 1,
                    rec.component.manufacturer,
                    rec.component.part_number,
                    rec.confidence * 100.0
                );
            }
        }
        Err(e) => println!("   ⚠️  Recommendation failed (expected without Ollama): {}", e),
    }
    println!();

    // Demonstrate embedding functionality (this works without Ollama)
    println!("3. 🧠 Vector Embedding Similarity Search");
    let ollama_client_embed = OpenCircuitOllamaClient::new();
    let mut embedding_engine = ComponentEmbeddingEngine::new(ollama_client_embed).await?;
    
    let components = create_sample_components();
    
    // This uses the simplified hash-based embedding which works without Ollama
    match embedding_engine.find_similar_components_by_requirements(
        "10k ohm resistor for voltage divider",
        &components,
        3
    ).await {
        Ok(matches) => {
            println!("   Found {} similar components:", matches.len());
            for (i, m) in matches.iter().enumerate() {
                println!("   {}. {} {} - Similarity: {:.1}%", 
                    i + 1,
                    m.component.manufacturer,
                    m.component.part_number,
                    m.similarity * 100.0
                );
                println!("      Reason: {}", m.match_reason);
            }
        }
        Err(e) => println!("   ⚠️  Similarity search failed: {}", e),
    }
    println!();

    // Show cache functionality
    println!("4. 💾 Embedding Cache Statistics");
    let (count, memory) = embedding_engine.cache_stats();
    println!("   Cached embeddings: {}", count);
    println!("   Estimated memory usage: {} bytes", memory);
    println!();

    println!("✅ Component Recommendation System Demonstration Complete!");
    println!("\n📝 Notes:");
    println!("   - This demo uses simplified embeddings that work without Ollama");
    println!("   - For full AI functionality, install and run Ollama with qwen2.5:0.5b model");
    println!("   - The system supports budget constraints, performance priorities, and more");
    println!("   - Vector embeddings enable semantic component search and similarity matching");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    demonstrate_component_recommendations().await
}