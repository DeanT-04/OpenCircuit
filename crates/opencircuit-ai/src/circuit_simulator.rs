//! Circuit simulation interface
//! Provides basic simulation capabilities for generated circuits

use crate::ollama_client::OpenCircuitOllamaClient;
use ollama_rs::generation::completion::request::GenerationRequest;
use opencircuit_core::circuit::{Netlist, ValidationReport};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SimulationError {
    #[error("Simulation failed: {0}")]
    SimulationFailed(String),
    
    #[error("Invalid circuit: {0}")]
    InvalidCircuit(String),
    
    #[error("Analysis error: {0}")]
    AnalysisError(String),
    
    #[error("Missing analysis type: {0}")]
    MissingAnalysis(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationRequest {
    pub netlist: Netlist,
    pub analysis_type: AnalysisType,
    pub parameters: SimulationParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisType {
    DC,
    AC {
        start_freq: f64,
        end_freq: f64,
        points_per_decade: usize,
    },
    Transient {
        start_time: f64,
        end_time: f64,
        step_size: f64,
    },
    OperatingPoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationParameters {
    pub temperature: f64,
    pub nominal_temperature: f64,
    pub gmin: f64,
    pub reltol: f64,
    pub abstol: f64,
    pub vntol: f64,
    pub max_iterations: usize,
}

impl Default for SimulationParameters {
    fn default() -> Self {
        Self {
            temperature: 27.0,
            nominal_temperature: 27.0,
            gmin: 1e-12,
            reltol: 1e-3,
            abstol: 1e-12,
            vntol: 1e-6,
            max_iterations: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub success: bool,
    pub analysis_type: AnalysisType,
    pub node_voltages: HashMap<String, f64>,
    pub branch_currents: HashMap<String, f64>,
    pub power_consumption: f64,
    pub warnings: Vec<String>,
    pub raw_output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ACAnalysisResult {
    pub frequencies: Vec<f64>,
    pub node_voltages: HashMap<String, Vec<ComplexValue>>,
    pub phase_response: HashMap<String, Vec<f64>>,
    pub magnitude_response: HashMap<String, Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexValue {
    pub real: f64,
    pub imag: f64,
}

impl ComplexValue {
    pub fn magnitude(&self) -> f64 {
        (self.real.powi(2) + self.imag.powi(2)).sqrt()
    }

    pub fn phase(&self) -> f64 {
        self.imag.atan2(self.real)
    }
}

pub struct CircuitSimulator {
    ollama_client: OpenCircuitOllamaClient,
}

impl CircuitSimulator {
    pub fn new(ollama_client: OpenCircuitOllamaClient) -> Self {
        Self { ollama_client }
    }

    pub async fn simulate(&self, request: SimulationRequest) -> Result<SimulationResult, SimulationError> {
        // First validate the circuit
        let validator = opencircuit_core::circuit::CircuitValidator::new();
        let validation_report = validator.validate(&request.netlist);

        if !validation_report.is_valid {
            return Err(SimulationError::InvalidCircuit(
                validation_report.errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; ")
            ));
        }

        // Generate simulation prompt
        let prompt = self.build_simulation_prompt(&request);

        // Use Ollama for simulation guidance
        let full_prompt = format!("You are a circuit simulation expert. Analyze the given SPICE netlist and provide simulation results. Return structured JSON with node voltages, branch currents, and power calculations.\n\nUser: {}", prompt);

        match self.ollama_client.complete(&full_prompt).await {
            Ok(response) => {
                let simulation_result = self.parse_simulation_response(&response, &request.analysis_type)?;
                Ok(simulation_result)
            },
            Err(e) => Err(SimulationError::SimulationFailed(format!("AI service error: {}", e)))
        }
    }

    fn build_simulation_prompt(&self, request: &SimulationRequest) -> String {
        let mut prompt = String::new();
        
        prompt.push_str("Perform circuit simulation for the following netlist:\n\n");
        prompt.push_str(&request.netlist.to_spice());
        prompt.push_str("\n\n");
        
        match &request.analysis_type {
            AnalysisType::DC => {
                prompt.push_str("Analysis Type: DC Operating Point\n");
                prompt.push_str("Calculate node voltages, branch currents, and power consumption.\n");
            },
            AnalysisType::AC { start_freq, end_freq, points_per_decade } => {
                prompt.push_str(&format!(
                    "Analysis Type: AC Analysis\nFrequency range: {} Hz to {} Hz\nPoints per decade: {}\n",
                    start_freq, end_freq, points_per_decade
                ));
            },
            AnalysisType::Transient { start_time, end_time, step_size } => {
                prompt.push_str(&format!(
                    "Analysis Type: Transient Analysis\nTime range: {}s to {}s\nStep size: {}s\n",
                    start_time, end_time, step_size
                ));
            },
            AnalysisType::OperatingPoint => {
                prompt.push_str("Analysis Type: Operating Point\n");
                prompt.push_str("Calculate DC operating point voltages and currents.\n");
            }
        }

        prompt.push_str("\nReturn results in JSON format with the following structure:\n");
        prompt.push_str("{\n");
        prompt.push_str("  \"success\": true/false,\n");
        prompt.push_str("  \"node_voltages\": {\"node_name\": voltage_value},\n");
        prompt.push_str("  \"branch_currents\": {\"component_name\": current_value},\n");
        prompt.push_str("  \"power_consumption\": total_power,\n");
        prompt.push_str("  \"warnings\": [\"warning1\", \"warning2\"]\n");
        prompt.push_str("}\n");

        prompt
    }

    fn parse_simulation_response(&self, response: &str, analysis_type: &AnalysisType) -> Result<SimulationResult, SimulationError> {
        // Try to parse as JSON first
        if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(response) {
            let node_voltages = self.extract_node_voltages(&json_data)?;
            let branch_currents = self.extract_branch_currents(&json_data)?;
            let power_consumption = self.extract_power_consumption(&json_data)?;
            let warnings = self.extract_warnings(&json_data)?;

            Ok(SimulationResult {
                success: true,
                analysis_type: analysis_type.clone(),
                node_voltages,
                branch_currents,
                power_consumption,
                warnings,
                raw_output: Some(response.to_string()),
            })
        } else {
            // Fallback to text parsing if JSON parsing fails
            self.parse_text_response(response, analysis_type)
        }
    }

    fn extract_node_voltages(&self, json_data: &serde_json::Value) -> Result<HashMap<String, f64>, SimulationError> {
        let mut voltages = HashMap::new();
        
        if let Some(node_voltages) = json_data.get("node_voltages").and_then(|v| v.as_object()) {
            for (node, voltage) in node_voltages {
                if let Some(v) = voltage.as_f64() {
                    voltages.insert(node.clone(), v);
                }
            }
        }

        Ok(voltages)
    }

    fn extract_branch_currents(&self, json_data: &serde_json::Value) -> Result<HashMap<String, f64>, SimulationError> {
        let mut currents = HashMap::new();
        
        if let Some(branch_currents) = json_data.get("branch_currents").and_then(|v| v.as_object()) {
            for (component, current) in branch_currents {
                if let Some(c) = current.as_f64() {
                    currents.insert(component.clone(), c);
                }
            }
        }

        Ok(currents)
    }

    fn extract_power_consumption(&self, json_data: &serde_json::Value) -> Result<f64, SimulationError> {
        json_data.get("power_consumption")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| SimulationError::AnalysisError("Missing power consumption data".to_string()))
    }

    fn extract_warnings(&self, json_data: &serde_json::Value) -> Result<Vec<String>, SimulationError> {
        let mut warnings = Vec::new();
        
        if let Some(warning_array) = json_data.get("warnings").and_then(|v| v.as_array()) {
            for warning in warning_array {
                if let Some(w) = warning.as_str() {
                    warnings.push(w.to_string());
                }
            }
        }

        Ok(warnings)
    }

    fn parse_text_response(&self, response: &str, analysis_type: &AnalysisType) -> Result<SimulationResult, SimulationError> {
        // Simple text parsing fallback
        let mut node_voltages = HashMap::new();
        let branch_currents = HashMap::new();
        let power_consumption = 0.0;
        let warnings = Vec::new();

        // Basic parsing - this would need to be enhanced based on actual response format
        for line in response.lines() {
            let line = line.trim();
            if line.contains("Node") && line.contains("=") {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() == 2 {
                    let node = parts[0].replace("Node", "").trim().to_string();
                    let voltage = parts[1].trim().parse::<f64>().unwrap_or(0.0);
                    node_voltages.insert(node, voltage);
                }
            }
        }

        Ok(SimulationResult {
            success: true,
            analysis_type: analysis_type.clone(),
            node_voltages,
            branch_currents,
            power_consumption,
            warnings,
            raw_output: Some(response.to_string()),
        })
    }

    pub async fn quick_analysis(&self, netlist: &Netlist) -> Result<ValidationReport, SimulationError> {
        let validator = opencircuit_core::circuit::CircuitValidator::new();
        let report = validator.validate(netlist);
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencircuit_core::circuit::{Component, ComponentType, Netlist};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_simulation_request_building() {
        let mut netlist = Netlist::new("Test Circuit".to_string());
        
        netlist.components.push(Component {
            name: "V1".to_string(),
            component_type: ComponentType::VoltageSource,
            nodes: vec!["1".to_string(), "0".to_string()],
            value: "12".to_string(),
            model: None,
            parameters: HashMap::new(),
        });

        let request = SimulationRequest {
            netlist,
            analysis_type: AnalysisType::DC,
            parameters: SimulationParameters::default(),
        };

        let simulator = CircuitSimulator::new(OllamaClient::new("http://localhost:11434".to_string()));
        let prompt = simulator.build_simulation_prompt(&request);
        
        assert!(prompt.contains("Perform circuit simulation"));
        assert!(prompt.contains("V1"));
    }

    #[test]
    fn test_complex_value_operations() {
        let complex = ComplexValue { real: 3.0, imag: 4.0 };
        assert_eq!(complex.magnitude(), 5.0);
        assert_eq!(complex.phase().round(), 0.927f64.round());
    }
}