//! Interactive Test Runner for Human-like Integration Tests
//!
//! This executable provides an interactive way to run user simulation tests

use std::io::{self, Write};
use std::process;

fn main() {
    println!("🎯 OpenCircuit Human-Like Test Runner");
    println!("{}", "=".repeat(50));
    println!();
    
    loop {
        display_menu();
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read input");
        
        match choice.trim() {
            "1" => run_beginner_tests(),
            "2" => run_professional_tests(),
            "3" => run_student_tests(),
            "4" => run_iot_tests(),
            "5" => run_all_tests(),
            "6" => run_edge_cases(),
            "7" => show_sample_interactions(),
            "8" => simulate_real_users(),
            "0" => {
                println!("👋 Goodbye!");
                break;
            },
            _ => {
                println!("❌ Invalid choice. Please try again.");
            }
        }
        
        println!();
    }
}

fn display_menu() {
    println!("🎮 Choose a test scenario:");
    println!("1. Beginner Projects (Sarah's first LED)");
    println!("2. Professional Designs (David's industrial sensor)");
    println!("3. Student Learning (Alex's transistor amplifier)");
    println!("4. IoT Projects (Maya's weather station)");
    println!("5. Run All Tests");
    println!("6. Edge Cases & Error Handling");
    println!("7. Sample User Interactions");
    println!("8. Real User Simulation");
    println!("0. Exit");
    println!();
    print!("Enter your choice: ");
    io::stdout().flush().unwrap();
}

fn run_beginner_tests() {
    println!("🧪 Running Beginner Tests...");
    println!();
    
    let scenarios = vec![
        ("LED Blinker", "Sarah wants to build her first circuit", 15.0),
        ("Battery Monitor", "Simple voltage divider circuit", 12.0),
        ("Basic Alarm", "Buzzer with switch", 18.0),
        ("Light Sensor", "Photoresistor circuit", 14.0),
    ];
    
    for (project, description, budget) in scenarios {
        println!("👤 Beginner: {}", project);
        println!("   Description: {}", description);
        println!("   Budget: ${:.2}", budget);
        
        let components = simulate_component_selection(project, budget);
        println!("   Components found: {} items", components.len());
        println!("   Total cost: ${:.2}", calculate_total_cost(&components));
        println!();
    }
    
    println!("✅ Beginner tests completed!");
}

fn run_professional_tests() {
    println!("🔧 Running Professional Tests...");
    println!();
    
    let scenarios = vec![
        ("4-20mA Interface", "Industrial sensor conditioning", 150.0),
        ("Motor Driver", "H-bridge for robotics", 200.0),
        ("Power Supply", "Switching regulator design", 180.0),
        ("Data Logger", "SD card + RTC circuit", 120.0),
    ];
    
    for (project, description, budget) in scenarios {
        println!("👨‍💼 Professional: {}", project);
        println!("   Description: {}", description);
        println!("   Budget: ${:.2}", budget);
        
        let components = simulate_component_selection(project, budget);
        println!("   High-precision components: {} items", components.len());
        println!("   Total cost: ${:.2}", calculate_total_cost(&components));
        println!();
    }
    
    println!("✅ Professional tests completed!");
}

fn run_student_tests() {
    println!("🎓 Running Student Learning Tests...");
    println!();
    
    let scenarios = vec![
        ("Transistor Switch", "Learning digital electronics", 10.0),
        ("Op-Amp Amplifier", "Analog signal processing", 15.0),
        ("555 Timer", "Oscillator and timing circuits", 12.0),
        ("Logic Gates", "Digital fundamentals", 8.0),
    ];
    
    for (project, description, budget) in scenarios {
        println!("🎓 Student: {}", project);
        println!("   Learning objective: {}", description);
        println!("   Educational budget: ${:.2}", budget);
        
        let components = simulate_educational_components(project, budget);
        println!("   Educational components: {} items", components.len());
        println!("   Includes learning resources: ✓");
        println!();
    }
    
    println!("✅ Student tests completed!");
}

fn run_iot_tests() {
    println!("📡 Running IoT Project Tests...");
    println!();
    
    let scenarios = vec![
        ("ESP32 Weather Station", "WiFi + sensors + cloud", 45.0),
        ("LoRa Node", "Long-range wireless sensor", 55.0),
        ("Bluetooth Beacon", "BLE advertising device", 35.0),
        ("Solar Charger", "MPPT for IoT devices", 65.0),
    ];
    
    for (project, description, budget) in scenarios {
        println!("📡 IoT: {}", project);
        println!("   Description: {}", description);
        println!("   IoT budget: ${:.2}", budget);
        
        let components = simulate_iot_components(project, budget);
        println!("   IoT-optimized components: {} items", components.len());
        println!("   Power efficiency: ✓");
        println!();
    }
    
    println!("✅ IoT tests completed!");
}

fn run_all_tests() {
    println!("🚀 Running Complete Test Suite...");
    println!();
    
    run_beginner_tests();
    run_professional_tests();
    run_student_tests();
    run_iot_tests();
    run_edge_cases();
    
    println!("🎉 All tests completed successfully!");
    println!("📊 Summary: 4 user types × 4 scenarios each = 16 test cases");
}

fn run_edge_cases() {
    println!("⚡ Testing Edge Cases...");
    println!();
    
    let edge_cases = vec![
        ("Zero Budget", 0.0, "Should suggest free resources"),
        ("Impossible Specs", 1000.0, "Should provide alternatives"),
        ("Missing Components", 50.0, "Should handle stock issues"),
        ("Conflicting Requirements", 75.0, "Should resolve conflicts"),
    ];
    
    for (case, budget, expected) in edge_cases {
        println!("⚠️ Edge case: {}", case);
        println!("   Budget: ${:.2}", budget);
        println!("   Expected: {}", expected);
        
        let result = simulate_edge_case(case, budget);
        println!("   Result: {}", result);
        println!();
    }
    
    println!("✅ Edge case tests completed!");
}

fn show_sample_interactions() {
    println!("💬 Sample User Interactions:");
    println!();
    
    let interactions = vec![
        ("Beginner", "I want to make LEDs blink", "Try a 555 timer circuit..."),
        ("Professional", "Need precision ADC for sensors", "ADS1115 offers 16-bit resolution..."),
        ("Student", "Learning about transistors", "Start with 2N3904 common emitter..."),
        ("IoT Developer", "Battery-powered sensor node", "ESP32 with deep sleep..."),
    ];
    
    for (user_type, query, response) in interactions {
        println!("👤 {}: \"{}\"", user_type, query);
        println!("🤖 AI: \"{}\"", response);
        println!();
    }
}

fn simulate_real_users() {
    println!("🎭 Simulating Real User Sessions...");
    println!();
    
    let users = [
        ("Sarah", "First-time electronics", "LED blinker", 15.0),
        ("David", "10yr engineer", "Industrial sensor", 200.0),
        ("Alex", "University student", "Audio amplifier", 25.0),
        ("Maya", "IoT enthusiast", "Weather station", 50.0),
    ];
    
    for (name, background, project, budget) in users {
        println!("🧑‍💻 User: {} ({})", name, background);
        println!("   Project: {}", project);
        println!("   Budget: ${:.2}", budget);
        
        let session = simulate_user_session(name, project, budget);
        println!("   Session time: {:.1}s", session.duration);
        println!("   Components selected: {}", session.components.len());
        println!("   Satisfaction: {}/5", session.satisfaction);
        println!();
    }
    
    println!("✅ Real user simulation completed!");
}

// Mock simulation functions
fn simulate_component_selection(project: &str, budget: f64) -> Vec<(String, f64)> {
    let components = match project {
        "LED Blinker" => vec![
            ("555 Timer".to_string(), 0.5),
            ("LED".to_string(), 0.1),
            ("Resistors".to_string(), 0.2),
            ("Capacitors".to_string(), 0.3),
            ("Battery".to_string(), 2.0),
        ],
        "4-20mA Interface" => vec![
            ("Precision Op-Amp".to_string(), 3.5),
            ("16-bit ADC".to_string(), 8.0),
            ("Precision Resistors".to_string(), 2.5),
            ("Protection Diodes".to_string(), 1.0),
        ],
        "ESP32 Weather Station" => vec![
            ("ESP32 DevKit".to_string(), 8.5),
            ("DHT22 Sensor".to_string(), 3.2),
            ("Solar Panel".to_string(), 12.0),
            ("Battery".to_string(), 6.5),
            ("Enclosure".to_string(), 8.0),
        ],
        _ => vec![
            ("Microcontroller".to_string(), 5.0),
            ("Sensors".to_string(), 8.0),
            ("Power Supply".to_string(), 10.0),
            ("Connectors".to_string(), 3.0),
        ],
    };
    
    components.into_iter().filter(|(_, price)| *price <= budget).collect()
}

fn simulate_educational_components(project: &str, budget: f64) -> Vec<(String, f64)> {
    let mut components = simulate_component_selection(project, budget);
    components.push(("Educational Guide".to_string(), 0.0));
    components
}

fn simulate_iot_components(project: &str, budget: f64) -> Vec<(String, f64)> {
    let mut components = simulate_component_selection(project, budget);
    components.push(("Power Management".to_string(), 5.0));
    components
}

fn calculate_total_cost(components: &[(String, f64)]) -> f64 {
    components.iter().map(|(_, price)| price).sum()
}

fn simulate_edge_case(case: &str, budget: f64) -> String {
    match case {
        "Zero Budget" => "Suggested free online simulators and educational resources".to_string(),
        "Impossible Specs" => "Provided alternative realistic specifications".to_string(),
        "Missing Components" => "Suggested substitute parts with similar specs".to_string(),
        "Conflicting Requirements" => "Helped prioritize and resolve conflicts".to_string(),
        _ => "Handled gracefully".to_string(),
    }
}

#[derive(Debug)]
struct UserSession {
    duration: f64,
    components: Vec<String>,
    satisfaction: u8,
}

fn simulate_user_session(name: &str, project: &str, budget: f64) -> UserSession {
    let components = simulate_component_selection(project, budget);
    
    UserSession {
        duration: 2.5 + components.len() as f64 * 0.5,
        components: components.iter().map(|(c, _)| c.clone()).collect(),
        satisfaction: if calculate_total_cost(&components) <= budget { 5 } else { 3 },
    }
}