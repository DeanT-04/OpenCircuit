//! Simulation results and data structures

use crate::analysis::AnalysisType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete simulation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResults {
    /// Type of analysis performed
    pub analysis_type: AnalysisType,
    /// Analysis data
    pub data: AnalysisData,
    /// Simulation metadata
    pub metadata: HashMap<String, String>,
    /// Warnings generated during simulation
    pub warnings: Vec<String>,
}

/// Analysis data variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisData {
    /// DC analysis results
    DC(DCResults),
    /// AC analysis results
    AC(ACResults),
    /// Transient analysis results
    Transient(TransientResults),
    /// Raw output data (for debugging)
    Raw(Vec<String>),
}

/// DC analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DCResults {
    /// Operating point voltages (node -> voltage)
    pub node_voltages: HashMap<String, f64>,
    /// Branch currents (branch -> current)
    pub branch_currents: HashMap<String, f64>,
    /// Power dissipation (component -> power)
    pub power_dissipation: HashMap<String, f64>,
    /// Sweep results (if DC sweep was performed)
    pub sweep_data: Option<SweepResults>,
}

/// AC analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ACResults {
    /// Frequency points
    pub frequencies: Vec<f64>,
    /// Complex voltage responses (node -> complex values)
    pub voltage_responses: HashMap<String, Vec<ComplexValue>>,
    /// Complex current responses (branch -> complex values)
    pub current_responses: HashMap<String, Vec<ComplexValue>>,
    /// Transfer functions
    pub transfer_functions: HashMap<String, TransferFunction>,
}

/// Transient analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransientResults {
    /// Time points
    pub time_points: Vec<f64>,
    /// Voltage waveforms (node -> voltage values)
    pub voltage_waveforms: HashMap<String, Vec<f64>>,
    /// Current waveforms (branch -> current values)
    pub current_waveforms: HashMap<String, Vec<f64>>,
    /// Power waveforms (component -> power values)
    pub power_waveforms: HashMap<String, Vec<f64>>,
}

/// Sweep analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepResults {
    /// Parameter values that were swept
    pub parameter_values: Vec<f64>,
    /// Results for each parameter value
    pub results: Vec<DCResults>,
}

/// Complex value for AC analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexValue {
    /// Real part
    pub real: f64,
    /// Imaginary part
    pub imaginary: f64,
}

/// Transfer function data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferFunction {
    /// Input node/branch
    pub input: String,
    /// Output node/branch
    pub output: String,
    /// Magnitude response
    pub magnitude: Vec<f64>,
    /// Phase response (radians)
    pub phase: Vec<f64>,
}

/// Simulation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetadata {
    /// Simulation start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Simulation duration (seconds)
    pub duration: f64,
    /// NgSpice version used
    pub ngspice_version: Option<String>,
    /// Number of nodes in circuit
    pub node_count: usize,
    /// Number of components in circuit
    pub component_count: usize,
    /// Convergence information
    pub convergence_info: ConvergenceInfo,
}

/// Convergence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceInfo {
    /// Number of iterations required
    pub iterations: u32,
    /// Final error value
    pub final_error: f64,
    /// Whether convergence was achieved
    pub converged: bool,
}

impl ComplexValue {
    /// Create a new complex value
    pub fn new(real: f64, imaginary: f64) -> Self {
        Self { real, imaginary }
    }
    
    /// Calculate magnitude
    pub fn magnitude(&self) -> f64 {
        (self.real * self.real + self.imaginary * self.imaginary).sqrt()
    }
    
    /// Calculate phase in radians
    pub fn phase(&self) -> f64 {
        self.imaginary.atan2(self.real)
    }
    
    /// Calculate phase in degrees
    pub fn phase_degrees(&self) -> f64 {
        self.phase() * 180.0 / std::f64::consts::PI
    }
    
    /// Convert to magnitude and phase representation
    pub fn to_polar(&self) -> (f64, f64) {
        (self.magnitude(), self.phase())
    }
    
    /// Create from magnitude and phase
    pub fn from_polar(magnitude: f64, phase: f64) -> Self {
        Self {
            real: magnitude * phase.cos(),
            imaginary: magnitude * phase.sin(),
        }
    }
}

impl SimulationResults {
    /// Create new simulation results
    pub fn new(analysis_type: AnalysisType, data: AnalysisData) -> Self {
        Self {
            analysis_type,
            data,
            metadata: HashMap::new(),
            warnings: Vec::new(),
        }
    }
    
    /// Create new simulation results with default DC analysis
    pub fn default_dc() -> Self {
        Self {
            analysis_type: AnalysisType::DC,
            data: AnalysisData::Raw(Vec::new()),
            metadata: HashMap::new(),
            warnings: Vec::new(),
        }
    }
    
    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    /// Add warning
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
    
    /// Add warnings from a vector
    pub fn add_warnings(&mut self, warnings: Vec<String>) {
        self.warnings.extend(warnings);
    }
    
    /// Add raw output data
    pub fn add_raw_output(&mut self, output: Vec<String>) {
        self.data = AnalysisData::Raw(output);
    }
    
    /// Add structured data
    pub fn add_data(&mut self, key: String, values: HashMap<String, f64>) {
        // This is a placeholder for more sophisticated data handling
        self.metadata.insert(key, format!("{:?}", values));
    }
    
    /// Check if simulation was successful
    pub fn is_successful(&self) -> bool {
        match &self.data {
            AnalysisData::Raw(output) => {
                // Check if output contains error indicators
                !output.iter().any(|line| {
                    line.to_lowercase().contains("error") || 
                    line.to_lowercase().contains("failed")
                })
            },
            _ => true, // Structured data implies successful parsing
        }
    }
    
    /// Get summary of results
    pub fn summary(&self) -> String {
        match &self.data {
            AnalysisData::DC(dc) => {
                format!("DC Analysis: {} nodes, {} branches", 
                    dc.node_voltages.len(), 
                    dc.branch_currents.len())
            },
            AnalysisData::AC(ac) => {
                format!("AC Analysis: {} frequency points, {} nodes", 
                    ac.frequencies.len(), 
                    ac.voltage_responses.len())
            },
            AnalysisData::Transient(tran) => {
                format!("Transient Analysis: {} time points, {} nodes", 
                    tran.time_points.len(), 
                    tran.voltage_waveforms.len())
            },
            AnalysisData::Raw(output) => {
                format!("Raw Output: {} lines", output.len())
            },
        }
    }
}

impl Default for ConvergenceInfo {
    fn default() -> Self {
        Self {
            iterations: 0,
            final_error: 0.0,
            converged: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex_value_operations() {
        let c = ComplexValue::new(3.0, 4.0);
        assert_eq!(c.magnitude(), 5.0);
        assert!((c.phase() - 0.9272952180016122).abs() < 1e-10);
        assert!((c.phase_degrees() - 53.13010235415598).abs() < 1e-10);
    }
    
    #[test]
    fn test_complex_value_polar() {
        let c = ComplexValue::from_polar(5.0, std::f64::consts::PI / 4.0);
        assert!((c.real - 3.5355339059327378).abs() < 1e-10);
        assert!((c.imaginary - 3.5355339059327378).abs() < 1e-10);
        
        let (mag, phase) = c.to_polar();
        assert!((mag - 5.0).abs() < 1e-10);
        assert!((phase - std::f64::consts::PI / 4.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_simulation_results_creation() {
        let results = SimulationResults::new(
            AnalysisType::DC,
            AnalysisData::Raw(vec!["Test output".to_string()])
        );
        
        assert_eq!(results.analysis_type, AnalysisType::DC);
        assert!(results.is_successful());
        assert_eq!(results.summary(), "Raw Output: 1 lines");
    }
}