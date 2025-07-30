//! SPICE netlist parsing and generation utilities
//! Provides safe handling of SPICE netlist formats

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetlistError {
    #[error("Invalid netlist syntax: {0}")]
    SyntaxError(String),
    #[error("Unknown component type: {0}")]
    UnknownComponent(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid value format: {0}")]
    InvalidValue(String),
    #[error("Node not found: {0}")]
    NodeNotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Netlist {
    pub title: String,
    pub components: Vec<Component>,
    pub connections: Vec<Connection>,
    pub analysis_commands: Vec<AnalysisCommand>,
    pub models: Vec<Model>,
    pub includes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub component_type: ComponentType,
    pub nodes: Vec<String>,
    pub value: String,
    pub model: Option<String>,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ComponentType {
    Resistor,
    Capacitor,
    Inductor,
    VoltageSource,
    CurrentSource,
    Diode,
    Bjt,
    Mosfet,
    OpAmp,
    Transformer,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from_node: String,
    pub to_node: String,
    pub net_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisCommand {
    Op,
    Dc {
        source: String,
        start: f64,
        stop: f64,
        step: f64,
    },
    Ac {
        analysis_type: AcType,
        points: usize,
        start_freq: f64,
        stop_freq: f64,
    },
    Tran {
        step: f64,
        stop: f64,
        start: Option<f64>,
        uic: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AcType {
    Dec,
    Oct,
    Lin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub name: String,
    pub model_type: String,
    pub parameters: HashMap<String, String>,
}

impl Netlist {
    pub fn new(title: String) -> Self {
        Self {
            title,
            components: Vec::new(),
            connections: Vec::new(),
            analysis_commands: Vec::new(),
            models: Vec::new(),
            includes: Vec::new(),
        }
    }

    pub fn from_spice(text: &str) -> Result<Self, NetlistError> {
        let mut netlist = Netlist::new("Parsed Netlist".to_string());
        let lines: Vec<&str> = text.lines().collect();
        
        for line in lines {
            let line = line.trim();
            if line.is_empty() || line.starts_with('*') {
                continue;
            }

            if line.starts_with('.') {
                netlist.parse_dot_command(line)?;
            } else {
                netlist.parse_component(line)?;
            }
        }

        Ok(netlist)
    }

    pub fn to_spice(&self) -> String {
        let mut spice = String::new();
        
        spice.push_str(&format!("* {}\n", self.title));
        spice.push_str("\n");

        // Add includes
        for include in &self.includes {
            spice.push_str(&format!(".include {}\n", include));
        }

        if !self.includes.is_empty() {
            spice.push_str("\n");
        }

        // Add models
        for model in &self.models {
            spice.push_str(&format!(".model {} {} ", model.name, model.model_type));
            for (key, value) in &model.parameters {
                spice.push_str(&format!("{}={} ", key, value));
            }
            spice.push_str("\n");
        }

        if !self.models.is_empty() {
            spice.push_str("\n");
        }

        // Add components
        for component in &self.components {
            spice.push_str(&component.to_spice());
            spice.push_str("\n");
        }

        spice.push_str("\n");

        // Add analysis commands
        for command in &self.analysis_commands {
            spice.push_str(&command.to_spice());
            spice.push_str("\n");
        }

        spice.push_str("\n.end\n");

        spice
    }

    fn parse_dot_command(&mut self, line: &str) -> Result<(), NetlistError> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        match parts[0].to_lowercase().as_str() {
            ".ac" => {
                if parts.len() >= 5 {
                    let analysis_type = match parts[1] {
                        "dec" => AcType::Dec,
                        "oct" => AcType::Oct,
                        "lin" => AcType::Lin,
                        _ => return Err(NetlistError::SyntaxError("Invalid AC analysis type".to_string())),
                    };
                    
                    let points = parts[2].parse().unwrap_or(10);
                    let start_freq = parts[3].parse().unwrap_or(1.0);
                    let stop_freq = parts[4].parse().unwrap_or(1e6);

                    self.analysis_commands.push(AnalysisCommand::Ac {
                        analysis_type,
                        points,
                        start_freq,
                        stop_freq,
                    });
                }
            }
            ".dc" => {
                if parts.len() >= 5 {
                    let source = parts[1].to_string();
                    let start = parts[2].parse().unwrap_or(0.0);
                    let stop = parts[3].parse().unwrap_or(1.0);
                    let step = parts[4].parse().unwrap_or(0.1);

                    self.analysis_commands.push(AnalysisCommand::Dc {
                        source,
                        start,
                        stop,
                        step,
                    });
                }
            }
            ".tran" => {
                if parts.len() >= 3 {
                    let step = parts[1].parse().unwrap_or(1e-6);
                    let stop = parts[2].parse().unwrap_or(1e-3);
                    let start = if parts.len() >= 4 {
                        Some(parts[3].parse().unwrap_or(0.0))
                    } else {
                        None
                    };
                    let uic = parts.contains(&"uic");

                    self.analysis_commands.push(AnalysisCommand::Tran {
                        step,
                        stop,
                        start,
                        uic,
                    });
                }
            }
            ".op" => {
                self.analysis_commands.push(AnalysisCommand::Op);
            }
            ".include" => {
                if parts.len() >= 2 {
                    self.includes.push(parts[1].to_string());
                }
            }
            ".model" => {
                if parts.len() >= 3 {
                    let name = parts[1].to_string();
                    let model_type = parts[2].to_string();
                    let mut parameters = HashMap::new();

                    for param in parts.iter().skip(3) {
                        if let Some(eq_pos) = param.find('=') {
                            let key = &param[..eq_pos];
                            let value = &param[eq_pos + 1..];
                            parameters.insert(key.to_string(), value.to_string());
                        }
                    }

                    self.models.push(Model {
                        name,
                        model_type,
                        parameters,
                    });
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn parse_component(&mut self, line: &str) -> Result<(), NetlistError> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(NetlistError::SyntaxError("Invalid component definition".to_string()));
        }

        let name = parts[0].to_string();
        let component_type = match name.chars().next() {
            Some('R') | Some('r') => ComponentType::Resistor,
            Some('C') | Some('c') => ComponentType::Capacitor,
            Some('L') | Some('l') => ComponentType::Inductor,
            Some('V') | Some('v') => ComponentType::VoltageSource,
            Some('I') | Some('i') => ComponentType::CurrentSource,
            Some('D') | Some('d') => ComponentType::Diode,
            Some('Q') | Some('q') => ComponentType::Bjt,
            Some('M') | Some('m') => ComponentType::Mosfet,
            Some('X') | Some('x') => ComponentType::OpAmp,
            Some('T') | Some('t') => ComponentType::Transformer,
            _ => ComponentType::Custom(name.clone()),
        };

        let nodes = parts[1..parts.len() - 1].iter().map(|s| s.to_string()).collect();
        let value = parts.last().unwrap().to_string();

        let component = Component {
            name,
            component_type,
            nodes,
            value,
            model: None,
            parameters: HashMap::new(),
        };

        self.components.push(component);
        Ok(())
    }
}

impl Component {
    pub fn to_spice(&self) -> String {
        let mut spice = format!("{} ", self.name);
        
        for node in &self.nodes {
            spice.push_str(&format!("{} ", node));
        }

        if let Some(model) = &self.model {
            spice.push_str(&format!("{} ", model));
        }

        spice.push_str(&self.value);

        if !self.parameters.is_empty() {
            for (key, value) in &self.parameters {
                spice.push_str(&format!(" {}={}", key, value));
            }
        }

        spice
    }
}

impl AnalysisCommand {
    pub fn to_spice(&self) -> String {
        match self {
            AnalysisCommand::Op => ".op".to_string(),
            AnalysisCommand::Dc { source, start, stop, step } => {
                format!(".dc {} {} {} {}", source, start, stop, step)
            }
            AnalysisCommand::Ac { analysis_type, points, start_freq, stop_freq } => {
                let type_str = match analysis_type {
                    AcType::Dec => "dec",
                    AcType::Oct => "oct",
                    AcType::Lin => "lin",
                };
                format!(".ac {} {} {} {}", type_str, points, start_freq, stop_freq)
            }
            AnalysisCommand::Tran { step, stop, start, uic } => {
                let start_str = start.map(|s| s.to_string()).unwrap_or("0".to_string());
                let uic_str = if *uic { " uic" } else { "" };
                format!(".tran {} {} {}{}", step, stop, start_str, uic_str)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_netlist_parsing() {
        let spice = r#"
* Voltage Divider Circuit
V1 1 0 12V
R1 1 2 1k
R2 2 0 1k
.op
.end
"#;

        let netlist = Netlist::from_spice(spice).unwrap();
        assert_eq!(netlist.components.len(), 3);
        assert_eq!(netlist.analysis_commands.len(), 1);
    }

    #[test]
    fn test_netlist_generation() {
        let mut netlist = Netlist::new("Test Circuit".to_string());
        
        netlist.components.push(Component {
            name: "R1".to_string(),
            component_type: ComponentType::Resistor,
            nodes: vec!["1".to_string(), "2".to_string()],
            value: "1k".to_string(),
            model: None,
            parameters: HashMap::new(),
        });

        netlist.analysis_commands.push(AnalysisCommand::Op);

        let spice = netlist.to_spice();
        assert!(spice.contains("R1 1 2 1k"));
        assert!(spice.contains(".op"));
    }

    #[test]
    fn test_component_to_spice() {
        let component = Component {
            name: "C1".to_string(),
            component_type: ComponentType::Capacitor,
            nodes: vec!["1".to_string(), "0".to_string()],
            value: "10u".to_string(),
            model: None,
            parameters: HashMap::new(),
        };

        let spice = component.to_spice();
        assert_eq!(spice, "C1 1 0 10u");
    }

    #[test]
    fn test_analysis_command_to_spice() {
        let command = AnalysisCommand::Dc {
            source: "V1".to_string(),
            start: 0.0,
            stop: 10.0,
            step: 0.1,
        };

        let spice = command.to_spice();
        assert_eq!(spice, ".dc V1 0 10 0.1");
    }
}