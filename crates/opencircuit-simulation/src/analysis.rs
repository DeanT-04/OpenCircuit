//! Analysis types and configuration for circuit simulation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of circuit analysis supported
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnalysisType {
    /// DC operating point analysis
    DC,
    /// AC frequency analysis
    AC,
    /// Transient time-domain analysis
    Transient,
    /// DC sweep analysis
    DCSweep,
    /// Noise analysis
    Noise,
    /// Distortion analysis
    Distortion,
    /// Monte Carlo analysis
    MonteCarlo,
}

/// Analysis command configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisCommand {
    pub analysis_type: AnalysisType,
    pub parameters: HashMap<String, String>,
}

/// DC analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DCAnalysis {
    /// Include operating point calculation
    pub operating_point: bool,
    /// Sweep configuration
    pub sweep: Option<SweepConfig>,
    /// Temperature for analysis (Celsius)
    pub temperature: f64,
}

/// AC analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ACAnalysis {
    /// Analysis type (lin, oct, dec)
    pub sweep_type: ACSweepType,
    /// Number of points
    pub points: u32,
    /// Start frequency (Hz)
    pub start_freq: f64,
    /// Stop frequency (Hz)
    pub stop_freq: f64,
}

/// Transient analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransientAnalysis {
    /// Time step (seconds)
    pub time_step: f64,
    /// Stop time (seconds)
    pub stop_time: f64,
    /// Start time (seconds, optional)
    pub start_time: Option<f64>,
    /// Maximum time step (seconds, optional)
    pub max_time_step: Option<f64>,
}

/// AC sweep types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ACSweepType {
    Linear,
    Octave,
    Decade,
}

/// Parameter sweep configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepConfig {
    /// Parameter to sweep
    pub parameter: String,
    /// Start value
    pub start: f64,
    /// Stop value
    pub stop: f64,
    /// Step size or number of points
    pub step: f64,
    /// Sweep type
    pub sweep_type: SweepType,
}

/// Sweep types for parameter sweeps
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SweepType {
    Linear,
    Logarithmic,
    List(Vec<f64>),
}

impl AnalysisCommand {
    /// Create a DC operating point analysis
    pub fn dc_op() -> Self {
        Self {
            analysis_type: AnalysisType::DC,
            parameters: HashMap::new(),
        }
    }
    
    /// Create an AC analysis
    pub fn ac_analysis(sweep_type: ACSweepType, points: u32, start_freq: f64, stop_freq: f64) -> Self {
        let mut parameters = HashMap::new();
        parameters.insert("sweep_type".to_string(), format!("{:?}", sweep_type).to_lowercase());
        parameters.insert("points".to_string(), points.to_string());
        parameters.insert("start_freq".to_string(), start_freq.to_string());
        parameters.insert("stop_freq".to_string(), stop_freq.to_string());
        
        Self {
            analysis_type: AnalysisType::AC,
            parameters,
        }
    }
    
    /// Create a transient analysis
    pub fn transient_analysis(time_step: f64, stop_time: f64) -> Self {
        let mut parameters = HashMap::new();
        parameters.insert("time_step".to_string(), time_step.to_string());
        parameters.insert("stop_time".to_string(), stop_time.to_string());
        
        Self {
            analysis_type: AnalysisType::Transient,
            parameters,
        }
    }
    
    /// Convert to SPICE command string
    pub fn to_spice_command(&self) -> String {
        match self.analysis_type {
            AnalysisType::DC => {
                if let Some(sweep_param) = self.parameters.get("sweep_parameter") {
                    let start = self.parameters.get("start").map(|s| s.as_str()).unwrap_or("0");
                    let stop = self.parameters.get("stop").map(|s| s.as_str()).unwrap_or("1");
                    let step = self.parameters.get("step").map(|s| s.as_str()).unwrap_or("0.1");
                    format!(".dc {} {} {} {}", sweep_param, start, stop, step)
                } else {
                    ".op".to_string()
                }
            },
            
            AnalysisType::AC => {
                let sweep_type = self.parameters.get("sweep_type").map(|s| s.as_str()).unwrap_or("dec");
                let points = self.parameters.get("points").map(|s| s.as_str()).unwrap_or("10");
                let start_freq = self.parameters.get("start_freq").map(|s| s.as_str()).unwrap_or("1");
                let stop_freq = self.parameters.get("stop_freq").map(|s| s.as_str()).unwrap_or("1meg");
                format!(".ac {} {} {} {}", sweep_type, points, start_freq, stop_freq)
            },
            
            AnalysisType::Transient => {
                let time_step = self.parameters.get("time_step").map(|s| s.as_str()).unwrap_or("1n");
                let stop_time = self.parameters.get("stop_time").map(|s| s.as_str()).unwrap_or("1u");
                let start_time = self.parameters.get("start_time").map(|s| s.as_str()).unwrap_or("0");
                
                format!(".tran {} {} {}", time_step, stop_time, start_time)
            }
            
            _ => format!("* Unsupported analysis type: {:?}", self.analysis_type),
        }
    }
}

impl Default for DCAnalysis {
    fn default() -> Self {
        Self {
            operating_point: true,
            sweep: None,
            temperature: 27.0, // Room temperature
        }
    }
}

impl Default for ACAnalysis {
    fn default() -> Self {
        Self {
            sweep_type: ACSweepType::Decade,
            points: 10,
            start_freq: 1.0,
            stop_freq: 1e6,
        }
    }
}

impl Default for TransientAnalysis {
    fn default() -> Self {
        Self {
            time_step: 1e-9, // 1ns
            stop_time: 1e-6, // 1us
            start_time: None,
            max_time_step: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dc_op_command() {
        let cmd = AnalysisCommand::dc_op();
        assert_eq!(cmd.to_spice_command(), ".op");
    }
    
    #[test]
    fn test_ac_analysis_command() {
        let cmd = AnalysisCommand::ac_analysis(ACSweepType::Decade, 10, 1.0, 1e6);
        assert_eq!(cmd.to_spice_command(), ".ac decade 10 1 1000000");
    }
    
    #[test]
    fn test_transient_analysis_command() {
        let cmd = AnalysisCommand::transient_analysis(1e-9, 1e-6);
        assert_eq!(cmd.to_spice_command(), ".tran 0.000000001 0.000001 0");
    }
}