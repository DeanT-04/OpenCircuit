//! AI-powered circuit generation engine
//! Converts user requirements into valid SPICE netlists using LLM guidance

use crate::ollama_client::OpenCircuitOllamaClient;
use serde::{Deserialize, Serialize};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CircuitGenerationError {
    #[error("AI model error: {0}")]
    ModelError(String),
    #[error("Invalid circuit specification: {0}")]
    InvalidSpecification(String),
    #[error("Netlist generation failed: {0}")]
    NetlistGeneration(String),
    #[error("Circuit validation failed: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitRequirements {
    pub circuit_type: CircuitType,
    pub input_voltage: f64,
    pub output_voltage: Option<f64>,
    pub current_requirement: f64,
    pub frequency_range: Option<(f64, f64)>,
    pub constraints: Vec<Constraint>,
    pub preferred_components: Vec<String>,
    pub avoid_components: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CircuitType {
    PowerSupply,
    Amplifier,
    Filter,
    Oscillator,
    LogicGate,
    SensorInterface,
    MotorDriver,
    LedDriver,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constraint {
    SizeLimit { width: f64, height: f64 },
    CostLimit { max_cost: f64 },
    PowerLimit { max_power: f64 },
    TemperatureRange { min: f64, max: f64 },
    Precision { tolerance: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCircuit {
    pub netlist: String,
    pub components: Vec<ComponentSpec>,
    pub description: String,
    pub estimated_performance: PerformanceMetrics,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSpec {
    pub reference: String,
    pub part_number: String,
    pub value: String,
    pub footprint: String,
    pub description: String,
    pub cost_estimate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub efficiency: Option<f64>,
    pub bandwidth: Option<f64>,
    pub noise_level: Option<f64>,
    pub stability_margin: Option<f64>,
    pub estimated_cost: f64,
}

pub struct CircuitGenerator {
    ollama_client: OpenCircuitOllamaClient,
    system_prompt: String,
}

impl CircuitGenerator {
    pub fn new(ollama_client: OpenCircuitOllamaClient) -> Self {
        let system_prompt = r#"
You are an expert electronics engineer specializing in circuit design. 
Your task is to generate SPICE netlists based on user requirements.

Guidelines:
1. Always use standard SPICE syntax
2. Include proper component models when available
3. Add meaningful node names and component references
4. Consider real-world component values and tolerances
5. Include simulation commands when appropriate
6. Validate the circuit meets all specified requirements
7. Provide clear explanations for design choices

Respond with a JSON object containing:
- "netlist": the complete SPICE netlist
- "components": list of components with specifications
- "description": brief explanation of the circuit
- "estimated_performance": key performance metrics
- "warnings": any important considerations
"#.to_string();

        Self {
            ollama_client,
            system_prompt,
        }
    }

    pub async fn generate_circuit(
        &self,
        requirements: CircuitRequirements,
    ) -> Result<GeneratedCircuit, CircuitGenerationError> {
        let prompt = self.build_generation_prompt(&requirements);
        
        let full_prompt = format!("{}

User: {}", self.system_prompt, prompt);

        let response = self.ollama_client
            .complete(&full_prompt)
            .await
            .map_err(|e| CircuitGenerationError::ModelError(e.to_string()))?;

        self.parse_generated_circuit(&response)
    }

    fn build_generation_prompt(&self, requirements: &CircuitRequirements) -> String {
        let mut prompt = format!(
            "Generate a {} circuit with the following requirements:\n",
            match requirements.circuit_type {
                CircuitType::PowerSupply => "power supply",
                CircuitType::Amplifier => "amplifier",
                CircuitType::Filter => "filter",
                CircuitType::Oscillator => "oscillator",
                CircuitType::LogicGate => "logic gate",
                CircuitType::SensorInterface => "sensor interface",
                CircuitType::MotorDriver => "motor driver",
                CircuitType::LedDriver => "LED driver",
                CircuitType::Custom(ref desc) => desc.as_str(),
            }
        );

        prompt.push_str(&format!("- Input voltage: {}V\n", requirements.input_voltage));
        
        if let Some(output_voltage) = requirements.output_voltage {
            prompt.push_str(&format!("- Output voltage: {}V\n", output_voltage));
        }

        prompt.push_str(&format!("- Current requirement: {}A\n", requirements.current_requirement));

        if let Some((min_freq, max_freq)) = requirements.frequency_range {
            prompt.push_str(&format!("- Frequency range: {}Hz to {}Hz\n", min_freq, max_freq));
        }

        if !requirements.constraints.is_empty() {
            prompt.push_str("- Constraints:\n");
            for constraint in &requirements.constraints {
                match constraint {
                    Constraint::SizeLimit { width, height } => {
                        prompt.push_str(&format!("  - Maximum size: {}mm x {}mm\n", width, height));
                    }
                    Constraint::CostLimit { max_cost } => {
                        prompt.push_str(&format!("  - Maximum cost: ${}\n", max_cost));
                    }
                    Constraint::PowerLimit { max_power } => {
                        prompt.push_str(&format!("  - Maximum power: {}W\n", max_power));
                    }
                    Constraint::TemperatureRange { min, max } => {
                        prompt.push_str(&format!("  - Operating temperature: {}°C to {}°C\n", min, max));
                    }
                    Constraint::Precision { tolerance } => {
                        prompt.push_str(&format!("  - Precision tolerance: ±{}%\n", tolerance));
                    }
                }
            }
        }

        if !requirements.preferred_components.is_empty() {
            prompt.push_str(&format!("- Preferred components: {}\n", 
                requirements.preferred_components.join(", ")));
        }

        if !requirements.avoid_components.is_empty() {
            prompt.push_str(&format!("- Avoid components: {}\n", 
                requirements.avoid_components.join(", ")));
        }

        prompt.push_str("\nPlease provide a complete, functional circuit design.");
        prompt
    }

    fn parse_generated_circuit(&self, response: &str) -> Result<GeneratedCircuit, CircuitGenerationError> {
        // Try to parse JSON response
        if let Ok(circuit) = serde_json::from_str::<GeneratedCircuit>(response) {
            return Ok(circuit);
        }

        // Fallback: parse SPICE netlist from text
        self.parse_spice_netlist(response)
    }

    fn parse_spice_netlist(&self, text: &str) -> Result<GeneratedCircuit, CircuitGenerationError> {
        let lines: Vec<&str> = text.lines().collect();
        let mut netlist = String::new();
        let mut components = Vec::new();
        let mut description = String::new();
        
        let mut in_netlist = false;
        let mut in_description = false;

        for line in lines {
            let line = line.trim();
            
            if line.contains("* SPICE Netlist") || line.contains(".circuit") {
                in_netlist = true;
                continue;
            }
            
            if line.contains("* Description") || line.contains("## Description") {
                in_description = true;
                continue;
            }
            
            if in_netlist && !line.is_empty() {
                netlist.push_str(line);
                netlist.push('\n');
                
                // Parse component from netlist line
                if let Some(component) = self.parse_component_from_netlist(line) {
                    components.push(component);
                }
            }
            
            if in_description && !line.is_empty() {
                description.push_str(line);
                description.push(' ');
            }
        }

        if netlist.is_empty() {
            return Err(CircuitGenerationError::NetlistGeneration(
                "No valid netlist found in response".to_string()
            ));
        }

        Ok(GeneratedCircuit {
            netlist,
            components,
            description: description.trim().to_string(),
            estimated_performance: PerformanceMetrics {
                efficiency: None,
                bandwidth: None,
                noise_level: None,
                stability_margin: None,
                estimated_cost: 0.0,
            },
            warnings: vec![],
        })
    }

    fn parse_component_from_netlist(&self, line: &str) -> Option<ComponentSpec> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            return None;
        }

        let reference = parts[0].to_string();
        let value = parts.last()?.to_string();
        
        Some(ComponentSpec {
            reference: reference.clone(),
            part_number: "Generic".to_string(),
            value,
            footprint: "TH".to_string(),
            description: format!("Component {} from netlist", reference),
            cost_estimate: None,
        })
    }

    pub async fn validate_circuit(&self, circuit: &GeneratedCircuit) -> Result<(), CircuitGenerationError> {
        // Basic validation checks
        if circuit.netlist.trim().is_empty() {
            return Err(CircuitGenerationError::ValidationError(
                "Empty netlist provided".to_string()
            ));
        }

        // Check for required SPICE syntax
        if !circuit.netlist.contains(".end") && !circuit.netlist.contains(".END") {
            return Err(CircuitGenerationError::ValidationError(
                "Netlist missing .end statement".to_string()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ollama_client::OpenCircuitOllamaClient;

    #[tokio::test]
    async fn test_circuit_generation() {
        let client = OpenCircuitOllamaClient::new();
        let generator = CircuitGenerator::new(client);
        
        let requirements = CircuitRequirements {
            circuit_type: CircuitType::PowerSupply,
            input_voltage: 12.0,
            output_voltage: Some(5.0),
            current_requirement: 1.0,
            frequency_range: None,
            constraints: vec![Constraint::SizeLimit { width: 50.0, height: 30.0 }],
            preferred_components: vec!["LM7805".to_string()],
            avoid_components: vec!["switching".to_string()],
        };

        // This test would require a running Ollama server
        // For now, we'll just test the prompt generation
        let prompt = generator.build_generation_prompt(&requirements);
        assert!(prompt.contains("power supply"));
        assert!(prompt.contains("12V"));
        assert!(prompt.contains("5V"));
    }

    #[test]
    fn test_parse_component_from_netlist() {
        let generator = CircuitGenerator::new(OpenCircuitOllamaClient::new());
        
        let line = "R1 1 2 1k";
        let component = generator.parse_component_from_netlist(line).unwrap();
        
        assert_eq!(component.reference, "R1");
        assert_eq!(component.value, "1k");
    }

    #[test]
    fn test_parse_spice_netlist() {
        let generator = CircuitGenerator::new(OpenCircuitOllamaClient::new());
        
        let text = r#"
* SPICE Netlist for voltage divider
V1 1 0 12V
R1 1 2 1k
R2 2 0 1k
.end

* Description: Simple voltage divider
"#;

        let result = generator.parse_spice_netlist(text);
        assert!(result.is_ok());
        
        let circuit = result.unwrap();
        assert!(!circuit.netlist.is_empty());
        assert!(circuit.description.contains("voltage divider"));
    }
}