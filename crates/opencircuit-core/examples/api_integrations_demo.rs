//! Component API Integrations Demo
//! 
//! This example demonstrates how to use the OpenCircuit API integrations
//! to search for components across multiple suppliers (Octopart, DigiKey, Mouser).
//! 
//! Usage:
//! ```bash
//! cargo run --example api_integrations_demo
//! ```
//! 
//! Note: This demo uses mock configurations. For real API access, you'll need:
//! - Octopart API key
//! - DigiKey OAuth credentials
//! - Mouser API key

use opencircuit_core::{
    ApiManager, ApiConfig, OctopartConfig, DigiKeyConfig, MouserConfig,
    Component, ComponentCategory, ApiError
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 OpenCircuit API Integrations Demo");
    println!("=====================================\n");

    // Create API configuration
    let api_config = create_demo_config();
    
    // Initialize API manager
    let api_manager = ApiManager::new(api_config);
    
    println!("📋 API Manager Status:");
    print_api_status(&api_manager);
    
    // Demo 1: Search for components
    println!("\n🔍 Demo 1: Component Search");
    println!("---------------------------");
    demo_component_search(&api_manager).await;
    
    // Demo 2: Get component details
    println!("\n📊 Demo 2: Component Details");
    println!("----------------------------");
    demo_component_details(&api_manager).await;
    
    // Demo 3: Error handling
    println!("\n⚠️  Demo 3: Error Handling");
    println!("---------------------------");
    demo_error_handling(&api_manager).await;
    
    // Demo 4: Mock component creation
    println!("\n🧪 Demo 4: Mock Component Creation");
    println!("-----------------------------------");
    demo_mock_components();
    
    println!("\n✅ Demo completed successfully!");
    println!("\n💡 Next Steps:");
    println!("   1. Configure real API credentials in your config file");
    println!("   2. Set up rate limiting for production use");
    println!("   3. Implement caching strategies for better performance");
    println!("   4. Add error recovery and retry mechanisms");
    
    Ok(())
}

/// Create a demo configuration with mock credentials
fn create_demo_config() -> ApiConfig {
    ApiConfig {
        octopart: Some(OctopartConfig {
            enabled: true,
            api_key: "demo_octopart_key".to_string(),
            rate_limit: 100,
            cache_ttl: 3600,
        }),
        digikey: Some(DigiKeyConfig {
            enabled: true,
            client_id: "demo_digikey_client_id".to_string(),
            client_secret: "demo_digikey_secret".to_string(),
            sandbox: true,
            rate_limit: 50,
            cache_ttl: 3600,
        }),
        mouser: Some(MouserConfig {
            enabled: true,
            api_key: "demo_mouser_key".to_string(),
            rate_limit: 75,
            cache_ttl: 3600,
        }),
    }
}

/// Print the status of each API client
fn print_api_status(api_manager: &ApiManager) {
    println!("   Octopart: {}", if api_manager.octopart.is_some() { "✅ Enabled" } else { "❌ Disabled" });
    println!("   DigiKey:  {}", if api_manager.digikey.is_some() { "✅ Enabled" } else { "❌ Disabled" });
    println!("   Mouser:   {}", if api_manager.mouser.is_some() { "✅ Enabled" } else { "❌ Disabled" });
}

/// Demonstrate component search functionality
async fn demo_component_search(api_manager: &ApiManager) {
    println!("Searching for 'resistor 1k' across all APIs...");
    
    // Note: In a real implementation, this would make actual API calls
    // For demo purposes, we'll simulate the search process
    match api_manager.search_components("resistor 1k").await {
        Ok(components) => {
            println!("✅ Search completed successfully!");
            println!("   Found {} components", components.len());
            
            if components.is_empty() {
                println!("   📝 Note: No components found (using demo credentials)");
                println!("   💡 With real API keys, you would see actual search results");
            } else {
                for (i, component) in components.iter().take(3).enumerate() {
                    println!("   {}. {} - {} ({})", 
                        i + 1, 
                        component.part_number, 
                        component.manufacturer, 
                        format!("{:?}", component.category)
                    );
                }
            }
        }
        Err(e) => {
            println!("❌ Search failed: {}", e);
            println!("   📝 This is expected with demo credentials");
        }
    }
}

/// Demonstrate getting component details
async fn demo_component_details(api_manager: &ApiManager) {
    println!("Getting details for component 'RC0603FR-071KL'...");
    
    match api_manager.get_component_details("RC0603FR-071KL").await {
        Ok(Some(component)) => {
            println!("✅ Component details retrieved!");
            print_component_details(&component);
        }
        Ok(None) => {
            println!("❌ Component not found");
            println!("   📝 This is expected with demo credentials");
            println!("   💡 With real API keys, you would see detailed component information");
        }
        Err(e) => {
            println!("❌ Failed to get component details: {}", e);
            println!("   📝 This is expected with demo credentials");
            println!("   💡 With real API keys, you would see detailed component information");
        }
    }
}

/// Demonstrate error handling
async fn demo_error_handling(_api_manager: &ApiManager) {
    println!("Testing error handling scenarios...");
    
    // Simulate various error conditions
    let errors = vec![
        ApiError::AuthenticationFailed { 
            service: "octopart".to_string(), 
            reason: "Invalid API key provided".to_string() 
        },
        ApiError::RateLimitExceeded { service: "digikey".to_string() },
        ApiError::InvalidResponse("Malformed JSON response".to_string()),
        ApiError::NetworkError("Connection timeout".to_string()),
    ];
    
    for (i, error) in errors.iter().enumerate() {
        println!("   {}. {}: {}", i + 1, error_type_name(error), error);
    }
    
    println!("✅ Error handling scenarios demonstrated");
}

/// Demonstrate creating mock components for testing
fn demo_mock_components() {
    println!("Creating mock components for testing...");
    
    let components = vec![
        create_mock_resistor(),
        create_mock_capacitor(),
        create_mock_ic(),
    ];
    
    for component in components {
        print_component_summary(&component);
    }
    
    println!("✅ Mock components created successfully");
}

/// Create a mock resistor component
fn create_mock_resistor() -> Component {
    let mut component = Component::new(
        "RC0603FR-071KL".to_string(),
        "Yageo".to_string(),
        ComponentCategory::Resistors,
        "1kΩ ±1% 0.1W Thick Film Resistor 0603".to_string(),
    );
    
    let mut specs = HashMap::new();
    specs.insert("Resistance".to_string(), opencircuit_core::SpecValue::String("1kΩ".to_string()));
    specs.insert("Tolerance".to_string(), opencircuit_core::SpecValue::String("±1%".to_string()));
    specs.insert("Power".to_string(), opencircuit_core::SpecValue::String("0.1W".to_string()));
    specs.insert("Package".to_string(), opencircuit_core::SpecValue::String("0603".to_string()));
    component.specifications = specs;
    
    component
}

/// Create a mock capacitor component
fn create_mock_capacitor() -> Component {
    let mut component = Component::new(
        "CC0603KRX7R9BB104".to_string(),
        "Yageo".to_string(),
        ComponentCategory::Capacitors,
        "100nF ±10% 50V X7R Ceramic Capacitor 0603".to_string(),
    );
    
    let mut specs = HashMap::new();
    specs.insert("Capacitance".to_string(), opencircuit_core::SpecValue::String("100nF".to_string()));
    specs.insert("Tolerance".to_string(), opencircuit_core::SpecValue::String("±10%".to_string()));
    specs.insert("Voltage".to_string(), opencircuit_core::SpecValue::String("50V".to_string()));
    specs.insert("Dielectric".to_string(), opencircuit_core::SpecValue::String("X7R".to_string()));
    component.specifications = specs;
    
    component
}

/// Create a mock IC component
fn create_mock_ic() -> Component {
    let mut component = Component::new(
        "ATMEGA328P-PU".to_string(),
        "Microchip Technology".to_string(),
        ComponentCategory::IntegratedCircuits,
        "8-bit Microcontroller with 32KB Flash".to_string(),
    );
    
    let mut specs = HashMap::new();
    specs.insert("Architecture".to_string(), opencircuit_core::SpecValue::String("8-bit".to_string()));
    specs.insert("Flash Memory".to_string(), opencircuit_core::SpecValue::String("32KB".to_string()));
    specs.insert("RAM".to_string(), opencircuit_core::SpecValue::String("2KB".to_string()));
    specs.insert("Package".to_string(), opencircuit_core::SpecValue::String("DIP-28".to_string()));
    component.specifications = specs;
    
    component
}

/// Print detailed component information
fn print_component_details(component: &Component) {
    println!("   Part Number: {}", component.part_number);
    println!("   Manufacturer: {}", component.manufacturer);
    println!("   Category: {:?}", component.category);
    println!("   Description: {}", component.description);
    
    if !component.specifications.is_empty() {
        println!("   Specifications:");
        for (key, value) in &component.specifications {
            println!("     {}: {:?}", key, value);
        }
    }
    
    if let Some(price_info) = &component.price_info {
        println!("   Pricing: {} price breaks available", price_info.price_breaks.len());
    }
    
    if let Some(availability) = &component.availability {
        println!("   Availability: {} (Qty: {:?})", 
            if availability.in_stock { "In Stock" } else { "Out of Stock" },
            availability.quantity_available
        );
    }
}

/// Print a summary of a component
fn print_component_summary(component: &Component) {
    println!("   📦 {} - {} ({})", 
        component.part_number, 
        component.manufacturer,
        format!("{:?}", component.category)
    );
}

/// Get a human-readable error type name
fn error_type_name(error: &ApiError) -> &'static str {
    match error {
        ApiError::AuthenticationFailed { .. } => "Authentication Error",
        ApiError::RateLimitExceeded { .. } => "Rate Limit Error",
        ApiError::InvalidResponse(_) => "Response Error",
        ApiError::NetworkError(_) => "Network Error",
        ApiError::QuotaExceeded { .. } => "Quota Error",
        ApiError::ServiceUnavailable { .. } => "Service Error",
        ApiError::ConfigurationError(_) => "Configuration Error",
    }
}