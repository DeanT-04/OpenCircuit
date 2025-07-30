# Circuit Generation Engine - Rust Documentation

## Overview

The Circuit Generation Engine is a sophisticated AI-powered system for automatically generating electronic circuits based on natural language requirements. This engine is built in Rust and provides comprehensive circuit modeling, validation, and simulation capabilities.

## Architecture

The engine consists of several key components:

### 1. Circuit Generator (`circuit_generator.rs`)
AI-powered circuit creation from natural language specifications.

### 2. Circuit Simulator (`circuit_simulator.rs`)  
Basic simulation capabilities for generated circuits using AI models.

### 3. Validation Engine (`opencircuit-core/src/circuit/validation.rs`)
Comprehensive circuit validation and design rule checking.

### 4. Netlist Parser (`opencircuit-core/src/circuit/netlist.rs`)
SPICE netlist parsing and generation utilities.

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
opencircuit-ai = { path = "../opencircuit-ai" }
opencircuit-core = { path = "../opencircuit-core" }
```

### Basic Usage

```rust
use opencircuit_ai::circuit_generator::{CircuitGenerator, CircuitRequirements};
use opencircuit_ai::ollama_client::OllamaClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize AI client
    let client = OllamaClient::new("http://localhost:11434".to_string());
    let generator = CircuitGenerator::new(client);
    
    // Define circuit requirements
    let requirements = CircuitRequirements {
        circuit_type: "amplifier".to_string(),
        specifications: vec!["gain=10".to_string(), "bandwidth=1MHz".to_string()],
        constraints: vec!["single_supply".to_string(), "low_noise".to_string()],
        complexity: "intermediate".to_string(),
        ..Default::default()
    };
    
    // Generate circuit
    let circuit = generator.generate_circuit(requirements).await?;
    
    println!("Generated SPICE Netlist:");
    println!("{}", circuit.netlist.to_spice()?);
    
    Ok(())
}
```

## Circuit Requirements Specification

### Structure

```rust
pub struct CircuitRequirements {
    pub circuit_type: String,           // e.g., "amplifier", "filter", "oscillator"
    pub specifications: Vec<String>,    // Key parameters like "gain=20"
    pub constraints: Vec<String>,       // Design constraints like "low_power"
    pub complexity: String,              // "beginner", "intermediate", "advanced"
    pub preferred_components: Vec<String>, // Preferred part numbers
    pub application: Option<String>,     // Target application
    pub max_components: Option<usize>,   // Maximum component count
    pub operating_conditions: Option<String>, // Environmental conditions
}
```

### Examples

#### Audio Amplifier
```rust
let requirements = CircuitRequirements {
    circuit_type: "audio_amplifier".to_string(),
    specifications: vec![
        "gain=20dB".to_string(),
        "bandwidth=20Hz-20kHz".to_string(),
        "thd<0.1%".to_string(),
    ],
    constraints: vec![
        "single_supply_5V".to_string(),
        "low_power".to_string(),
        "surface_mount".to_string(),
    ],
    complexity: "intermediate".to_string(),
    preferred_components: vec!["LM358".to_string(), "TL072".to_string()],
    application: Some("portable_audio".to_string()),
    ..Default::default()
};
```

#### Sensor Interface
```rust
let requirements = CircuitRequirements {
    circuit_type: "sensor_interface".to_string(),
    specifications: vec![
        "input_range=0-5V".to_string(),
        "output_range=0-3.3V".to_string(),
        "noise<1mV".to_string(),
    ],
    constraints: vec![
        "battery_powered".to_string(),
        "low_power".to_string(),
    ],
    complexity: "beginner".to_string(),
    ..Default::default()
};
```

## Validation Engine

### Usage

```rust
use opencircuit_core::circuit::{CircuitValidator, Netlist};

fn validate_circuit() {
    let netlist = Netlist::from_spice(r#"
        * Simple RC Circuit
        V1 1 0 DC 5V
        R1 1 2 1k
        C1 2 0 1uF
        .end
    "#.to_string()).unwrap();
    
    let validator = CircuitValidator::new();
    let report = validator.validate(&netlist);
    
    if report.is_valid {
        println!("✅ Circuit is valid");
        println!("Components: {}", report.metrics.component_count);
        println!("Nodes: {}", report.metrics.node_count);
    } else {
        println!("❌ Circuit has errors:");
        for error in report.errors {
            println!("  - {}", error);
        }
    }
}
```

### Validation Rules

#### Electrical Rules
- **Ground Reference**: Must have connection to node 0
- **Floating Nodes**: All nodes must connect to ≥2 components
- **Short Circuits**: Detects direct shorts between sources
- **Component Values**: Validates value ranges

#### Design Rules
- **Naming Conflicts**: Unique component names
- **Connectivity**: Complete circuit paths
- **Power Analysis**: Missing power sources

## Circuit Simulation

### Basic Simulation

```rust
use opencircuit_ai::circuit_simulator::{CircuitSimulator, SimulationRequest, AnalysisType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let simulator = CircuitSimulator::new(client);
    
    let request = SimulationRequest {
        netlist: /* your netlist */,
        analysis_type: AnalysisType::DC,
        parameters: Default::default(),
    };
    
    let result = simulator.simulate(request).await?;
    
    for (node, voltage) in result.node_voltages {
        println!("Node {}: {:.3}V", node, voltage);
    }
    
    Ok(())
}
```

### Analysis Types

- **DC Analysis**: Operating point calculation
- **AC Analysis**: Frequency response
- **Transient**: Time-domain analysis
- **Operating Point**: DC bias calculation

## Netlist Format

### Supported Components

```spice
* Resistors
R1 1 2 1k

* Capacitors  
C1 1 2 1uF

* Inductors
L1 1 2 10mH

* Voltage Sources
V1 1 0 DC 12V
V2 1 0 AC 1V 1kHz

* Current Sources
I1 1 0 DC 1mA

* Diodes
D1 1 2 1N4148

* Transistors
Q1 3 2 1 2N3904
```

### Analysis Commands

```spice
.DC V1 0 5 0.1
.AC DEC 10 1 1MEG
.TRAN 1u 1m
.OP
```

## Error Handling

### Validation Errors

```rust
use opencircuit_core::circuit::ValidationError;

match validator.validate(&netlist) {
    Ok(report) => {
        if !report.is_valid {
            for error in report.errors {
                match error {
                    ValidationError::MissingGround(msg) => {
                        eprintln!("Ground connection missing: {}", msg);
                    }
                    ValidationError::InvalidValue(msg) => {
                        eprintln!("Component value error: {}", msg);
                    }
                    ValidationError::ShortCircuit(msg) => {
                        eprintln!("Short circuit detected: {}", msg);
                    }
                    _ => eprintln!("Validation error: {}", error),
                }
            }
        }
    }
    Err(e) => eprintln!("Validation failed: {}", e),
}
```

## Performance

### Benchmarks

| Operation | Small Circuit (<10 components) | Large Circuit (>100 components) |
|-----------|-------------------------------|--------------------------------|
| Validation | <1ms                          | <10ms                          |
| Generation | 2-5s (AI dependent)          | 5-15s (AI dependent)          |
| Simulation | 1-3s (AI dependent)          | 3-10s (AI dependent)          |

### Memory Usage
- Small circuits: ~1-5MB
- Large circuits: ~10-50MB
- Validation: Minimal additional memory

## Testing

### Unit Tests

```bash
# Run all circuit tests
cargo test --package opencircuit-ai --lib circuit_generator
cargo test --package opencircuit-core --lib circuit

# Run specific test modules
cargo test --lib circuit::validation
cargo test --lib circuit::netlist
```

### Integration Tests

```bash
# Run integration tests with AI service
cargo test --test circuit_integration

# Run with specific model
cargo test --test circuit_integration -- --model llama2
```

## Configuration

### AI Service Configuration

```rust
use opencircuit_ai::AiConfig;

let config = AiConfig {
    ollama_host: "http://localhost".to_string(),
    ollama_port: 11434,
    default_model: models::AiModel::Llama2,
    max_history: 100,
    timeout_seconds: 60,
    ..Default::default()
};
```

### Simulation Parameters

```rust
use opencircuit_ai::circuit_simulator::SimulationParameters;

let params = SimulationParameters {
    temperature: 27.0,
    nominal_temperature: 27.0,
    gmin: 1e-12,
    reltol: 1e-3,
    abstol: 1e-12,
    vntol: 1e-6,
    max_iterations: 100,
};
```

## API Reference

### CircuitGenerator

```rust
impl CircuitGenerator {
    /// Create new generator with Ollama client
    pub fn new(client: OllamaClient) -> Self;
    
    /// Generate circuit from requirements
    pub async fn generate_circuit(&self, requirements: CircuitRequirements) -> Result<GeneratedCircuit>;
    
    /// Validate generated circuit
    pub async fn validate_circuit(&self, circuit: &GeneratedCircuit) -> Result<ValidationReport>;
    
    /// Generate component suggestions
    pub async fn suggest_components(&self, requirements: &CircuitRequirements) -> Result<Vec<String>>;
}
```

### CircuitValidator

```rust
impl CircuitValidator {
    /// Create new validator with default rules
    pub fn new() -> Self;
    
    /// Validate netlist
    pub fn validate(&self, netlist: &Netlist) -> ValidationReport;
    
    /// Add custom validation rule
    pub fn add_rule(&mut self, rule: DesignRule);
    
    /// Set component value limits
    pub fn set_value_limits(&mut self, component_type: ComponentType, min: f64, max: f64);
}
```

## Examples Repository

For complete working examples, see:
- `examples/circuit_generation.rs`
- `examples/validation_workflow.rs`
- `examples/simulation_demo.rs`
- `tests/circuit_integration.rs`

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines on contributing to the circuit generation engine.

## License

MIT License - see [LICENSE](../LICENSE) for details.