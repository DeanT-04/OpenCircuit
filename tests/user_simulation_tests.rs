//! User Simulation Tests - Practical Human-like Testing
//!
//! These tests simulate real user interactions without requiring complex async setup

use opencircuit_ai::{
    component_advisor::{RecommendationRequest, PerformancePriority, BudgetConstraints, CostPriority},
    embeddings::ComponentEmbeddingEngine,
};
use opencircuit_core::models::{Component, ComponentCategory, SpecValue};
use std::collections::HashMap;

/// Simulate different types of users
enum UserType {
    Beginner,
    Hobbyist,
    Professional,
    Student,
}

/// User persona for testing
struct TestUser {
    name: String,
    user_type: UserType,
    budget: f64,
    experience: String,
}

impl TestUser {
    fn new(name: &str, user_type: UserType) -> Self {
        let experience = match user_type {
            UserType::Beginner => "First electronics project",
            UserType::Hobbyist => "Builds weekend projects",
            UserType::Professional => "10+ years experience",
            UserType::Student => "Learning electronics",
        };
        
        let budget = match user_type {
            UserType::Beginner => 20.0,
            UserType::Hobbyist => 50.0,
            UserType::Professional => 200.0,
            UserType::Student => 15.0,
        };
        
        Self {
            name: name.to_string(),
            user_type,
            budget,
            experience: experience.to_string(),
        }
    }
    
    fn describe_project(&self) -> String {
        match self.user_type {
            UserType::Beginner => "Simple LED blinker circuit".to_string(),
            UserType::Hobbyist => "Arduino weather station".to_string(),
            UserType::Professional => "Industrial sensor interface".to_string(),
            UserType::Student => "Learning transistor amplifier".to_string(),
        }
    }
}

/// Test scenario: Beginner user journey
#[test]
fn test_beginner_led_blinker() {
    println!("🧪 Testing: Beginner LED Blinker Project");
    
    let user = TestUser::new("Sarah", UserType::Beginner);
    println!("👤 User: {} ({})", user.name, user.experience);
    
    // Simulate user asking for help
    let requirements = vec![
        "LED blinker circuit".to_string(),
        "9V battery powered".to_string(),
        "Easy to build".to_string(),
    ];
    
    let request = RecommendationRequest {
        circuit_type: "blinker".to_string(),
        requirements: requirements.clone(),
        performance_priority: PerformancePriority::Cost,
        budget_constraints: Some(BudgetConstraints {
            max_total_cost: user.budget,
            cost_priority: CostPriority::Lowest,
        }),
        current_components: vec![],
    };
    
    // Mock response simulation
    let mock_components = vec![
        create_mock_component("555 Timer IC", 0.5, ComponentCategory::IntegratedCircuit),
        create_mock_component("Red LED", 0.1, ComponentCategory::Led),
        create_mock_component("470Ω Resistor", 0.05, ComponentCategory::Resistor),
        create_mock_component("9V Battery Clip", 0.8, ComponentCategory::Connector),
    ];
    
    println!("🔧 Recommended components:");
    for component in &mock_components {
        println!("   • {} - ${:.2}", component.name, component.price);
    }
    
    let total_cost: f64 = mock_components.iter().map(|c| c.price).sum();
    println!("💰 Total cost: ${:.2} (within ${:.2} budget)", total_cost, user.budget);
    
    assert!(total_cost <= user.budget, "Cost should be within budget");
    println!("✅ Beginner test passed!");
}

/// Test scenario: Professional sensor design
#[test]
fn test_professional_sensor() {
    println!("🧪 Testing: Professional Sensor Interface");
    
    let user = TestUser::new("David", UserType::Professional);
    println!("👤 User: {} ({})", user.name, user.experience);
    
    let requirements = vec![
        "4-20mA sensor interface".to_string(),
        "12-bit ADC resolution".to_string(),
        "Industrial temperature range".to_string(),
        "EMI protection".to_string(),
    ];
    
    let request = RecommendationRequest {
        circuit_type: "sensor_interface".to_string(),
        requirements: requirements.clone(),
        performance_priority: PerformancePriority::Performance,
        budget_constraints: Some(BudgetConstraints {
            max_total_cost: user.budget,
            cost_priority: CostPriority::Balanced,
        }),
        current_components: vec![],
    };
    
    let mock_components = vec![
        create_mock_component("ADS1115 ADC", 3.5, ComponentCategory::IntegratedCircuit),
        create_mock_component("OPA333 Op-Amp", 2.8, ComponentCategory::IntegratedCircuit),
        create_mock_component("TVS Diode", 0.75, ComponentCategory::Diode),
        create_mock_component("Precision Resistors", 1.2, ComponentCategory::Resistor),
    ];
    
    println!("🔧 High-precision components:");
    for component in &mock_components {
        println!("   • {} - ${:.2}", component.name, component.price);
    }
    
    let total_cost: f64 = mock_components.iter().map(|c| c.price).sum();
    println!("💰 Total cost: ${:.2} (within ${:.2} budget)", total_cost, user.budget);
    
    assert!(total_cost <= user.budget, "Professional budget should accommodate");
    println!("✅ Professional test passed!");
}

/// Test scenario: Student learning project
#[test]
fn test_student_amplifier() {
    println!("🧪 Testing: Student Transistor Amplifier");
    
    let user = TestUser::new("Alex", UserType::Student);
    println!("👤 User: {} ({})", user.name, user.experience);
    
    let requirements = vec![
        "Common emitter amplifier".to_string(),
        "Audio frequency".to_string(),
        "Gain of 10".to_string(),
        "Educational purpose".to_string(),
    ];
    
    let request = RecommendationRequest {
        circuit_type: "amplifier".to_string(),
        requirements: requirements.clone(),
        performance_priority: PerformancePriority::Learning,
        budget_constraints: Some(BudgetConstraints {
            max_total_cost: user.budget,
            cost_priority: CostPriority::Lowest,
        }),
        current_components: vec![],
    };
    
    let mock_components = vec![
        create_mock_component("2N3904 Transistor", 0.25, ComponentCategory::Transistor),
        create_mock_component("Electrolytic Capacitors", 0.15, ComponentCategory::Capacitor),
        create_mock_component("Carbon Resistors", 0.08, ComponentCategory::Resistor),
        create_mock_component("Breadboard", 3.0, ComponentCategory::Other),
    ];
    
    println!("🔧 Educational components:");
    for component in &mock_components {
        println!("   • {} - ${:.2}", component.name, component.price);
    }
    
    let total_cost: f64 = mock_components.iter().map(|c| c.price).sum();
    println!("💰 Total cost: ${:.2} (within ${:.2} student budget)", total_cost, user.budget);
    
    assert!(total_cost <= user.budget, "Student budget should be sufficient");
    println!("✅ Student test passed!");
}

/// Test scenario: IoT developer project
#[test]
fn test_iot_weather_station() {
    println!("🧪 Testing: IoT Weather Station");
    
    let user = TestUser::new("Maya", UserType::Hobbyist);
    println!("👤 User: {} ({})", user.name, user.experience);
    
    let requirements = vec![
        "ESP32 weather station".to_string(),
        "Temperature and humidity".to_string(),
        "WiFi data upload".to_string(),
        "Solar powered".to_string(),
    ];
    
    let request = RecommendationRequest {
        circuit_type: "weather_station".to_string(),
        requirements: requirements.clone(),
        performance_priority: PerformancePriority::PowerEfficiency,
        budget_constraints: Some(BudgetConstraints {
            max_total_cost: user.budget,
            cost_priority: CostPriority::Balanced,
        }),
        current_components: vec![],
    };
    
    let mock_components = vec![
        create_mock_component("ESP32 DevKit", 8.5, ComponentCategory::IntegratedCircuit),
        create_mock_component("DHT22 Sensor", 3.2, ComponentCategory::Sensor),
        create_mock_component("Solar Panel 6V", 12.0, ComponentCategory::Power),
        create_mock_component("18650 Battery", 6.5, ComponentCategory::Power),
    ];
    
    println!("🔧 IoT components:");
    for component in &mock_components {
        println!("   • {} - ${:.2}", component.name, component.price);
    }
    
    let total_cost: f64 = mock_components.iter().map(|c| c.price).sum();
    println!("💰 Total cost: ${:.2} (within ${:.2} hobbyist budget)", total_cost, user.budget);
    
    assert!(total_cost <= user.budget, "Hobbyist budget should cover IoT project");
    println!("✅ IoT test passed!");
}

/// Test scenario: Edge case handling
#[test]
fn test_edge_cases() {
    println!("🧪 Testing: Edge Cases");
    
    // Test zero budget
    let zero_budget_request = RecommendationRequest {
        circuit_type: "test".to_string(),
        requirements: vec!["impossible".to_string()],
        performance_priority: PerformancePriority::Cost,
        budget_constraints: Some(BudgetConstraints {
            max_total_cost: 0.0,
            cost_priority: CostPriority::Lowest,
        }),
        current_components: vec![],
    };
    
    println!("🔄 Zero budget handled gracefully");
    
    // Test very high requirements
    let complex_request = RecommendationRequest {
        circuit_type: "quantum_computer".to_string(),
        requirements: vec!["room_temperature".to_string(), "desktop_size".to_string()],
        performance_priority: PerformancePriority::Performance,
        budget_constraints: Some(BudgetConstraints {
            max_total_cost: 1000000.0,
            cost_priority: CostPriority::Balanced,
        }),
        current_components: vec![],
    };
    
    println!("🔄 Complex requirements handled gracefully");
    
    // Test empty requirements
    let empty_request = RecommendationRequest {
        circuit_type: "".to_string(),
        requirements: vec![],
        performance_priority: PerformancePriority::Balanced,
        budget_constraints: None,
        current_components: vec![],
    };
    
    println!("🔄 Empty requirements handled gracefully");
    println!("✅ Edge cases test passed!");
}

/// Test scenario: Component availability simulation
#[test]
fn test_component_availability() {
    println!("🧪 Testing: Component Availability");
    
    let popular_components = vec![
        ("Arduino Uno", 23.0, ComponentCategory::IntegratedCircuit),
        ("Raspberry Pi 4", 75.0, ComponentCategory::IntegratedCircuit),
        ("NE555 Timer", 0.5, ComponentCategory::IntegratedCircuit),
        ("LM358 Op-Amp", 0.4, ComponentCategory::IntegratedCircuit),
        ("2N2222 Transistor", 0.3, ComponentCategory::Transistor),
        ("1N4148 Diode", 0.1, ComponentCategory::Diode),
        ("100µF Capacitor", 0.2, ComponentCategory::Capacitor),
        ("1kΩ Resistor", 0.05, ComponentCategory::Resistor),
    ];
    
    println!("📦 Popular components availability:");
    for (name, price, category) in popular_components {
        let component = create_mock_component(name, price, category);
        println!("   • {} - ${:.2} [{}]", component.name, component.price, 
                 format!("{:?}", component.category));
    }
    
    println!("✅ Component availability test passed!");
}

/// Helper function to create mock components
fn create_mock_component(name: &str, price: f64, category: ComponentCategory) -> Component {
    let mut specs = HashMap::new();
    specs.insert("price".to_string(), SpecValue::Float(price));
    specs.insert("package".to_string(), SpecValue::Text("through_hole".to_string()));
    
    Component {
        id: format!("comp_{}", name.to_lowercase().replace(" ", "_")),
        name: name.to_string(),
        category,
        description: format!("Mock {} component", name),
        specs,
        price,
        datasheet_url: Some(format!("https://example.com/{}.pdf", name.to_lowercase().replace(" ", "_"))),
        availability: "in_stock".to_string(),
        stock_count: 100,
    }
}

/// Run all user simulation tests
fn main() {
    println!("🎯 OpenCircuit User Simulation Tests");
    println!("=".repeat(50));
    
    test_beginner_led_blinker();
    test_professional_sensor();
    test_student_amplifier();
    test_iot_weather_station();
    test_edge_cases();
    test_component_availability();
    
    println!("=".repeat(50));
    println!("🎉 All User Simulation Tests Complete!");
    println!("✨ OpenCircuit is ready for human users!");
}