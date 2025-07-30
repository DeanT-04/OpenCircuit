//! SPICE netlist parser and generator for OpenCircuit
//! 
//! This module provides functionality to:
//! - Generate SPICE netlists from Circuit objects
//! - Parse SPICE netlists into Circuit objects
//! - Handle various component types and their SPICE representations

use crate::errors::{Result, SimulationError};
use opencircuit_circuit::{Circuit, Component, ComponentType};
use std::collections::HashMap;
use std::fmt::Write;

/// SPICE netlist parser and generator
pub struct SpiceParser {
    component_counter: HashMap<ComponentType, u32>,
}

/// Netlist builder for constructing SPICE netlists
pub struct NetlistBuilder {
    title: String,
    components: Vec<String>,
    analysis_commands: Vec<String>,
    control_commands: Vec<String>,
}

impl SpiceParser {
    /// Create a new SPICE parser
    pub fn new() -> Self {
        Self {
            component_counter: HashMap::new(),
        }
    }
    
    /// Generate a SPICE netlist from a circuit
    pub fn generate_netlist(&mut self, circuit: &Circuit) -> Result<String> {
        let mut builder = NetlistBuilder::new("OpenCircuit Generated Circuit");
        
        // Add components to netlist
        for component in &circuit.components {
            let spice_line = self.component_to_spice(component)?;
            builder.add_component(spice_line);
        }
        
        // Add default analysis commands
        builder.add_analysis(".op"); // Operating point analysis
        builder.add_control(".end");
        
        Ok(builder.build())
    }
    
    /// Convert a component to SPICE format
    fn component_to_spice(&mut self, component: &Component) -> Result<String> {
        let component_id = self.get_component_id(&component.component_type);
        
        // Generate node assignments based on component type
        let (node1, node2, node3, node4, node5) = self.generate_node_assignments(&component.component_type);
        
        match &component.component_type {
            ComponentType::Resistor => {
                let value = component.value.as_ref()
                    .ok_or_else(|| SimulationError::InvalidComponent {
                        component: component.id.clone(),
                        reason: "Resistor missing value".to_string(),
                    })?;
                
                Ok(format!("R{} {} {} {}", 
                    component_id, node1, node2, value
                ))
            },
            
            ComponentType::Capacitor => {
                let value = component.value.as_ref()
                    .ok_or_else(|| SimulationError::InvalidComponent {
                        component: component.id.clone(),
                        reason: "Capacitor missing value".to_string(),
                    })?;
                
                Ok(format!("C{} {} {} {}", 
                    component_id, node1, node2, value
                ))
            },
            
            ComponentType::Inductor => {
                let value = component.value.as_ref()
                    .ok_or_else(|| SimulationError::InvalidComponent {
                        component: component.id.clone(),
                        reason: "Inductor missing value".to_string(),
                    })?;
                
                Ok(format!("L{} {} {} {}", 
                    component_id, node1, node2, value
                ))
            },
            
            ComponentType::VoltageSource => {
                let value = component.value.as_ref()
                    .ok_or_else(|| SimulationError::InvalidComponent {
                        component: component.id.clone(),
                        reason: "Voltage source missing value".to_string(),
                    })?;
                
                Ok(format!("V{} {} {} DC {}", 
                    component_id, node1, node2, value
                ))
            },
            
            ComponentType::CurrentSource => {
                let value = component.value.as_ref()
                    .ok_or_else(|| SimulationError::InvalidComponent {
                        component: component.id.clone(),
                        reason: "Current source missing value".to_string(),
                    })?;
                
                Ok(format!("I{} {} {} DC {}", 
                    component_id, node1, node2, value
                ))
            },
            
            ComponentType::Diode => {
                Ok(format!("D{} {} {} D1N4148", 
                    component_id, node1, node2
                ))
            },
            
            ComponentType::Transistor => {
                Ok(format!("Q{} {} {} {} 2N2222", 
                    component_id, node1, node2, node3
                ))
            },
            
            ComponentType::OpAmp => {
                Ok(format!("X{} {} {} {} {} {} LM741", 
                    component_id, node1, node2, node3, node4, node5
                ))
            },
        }
    }
    
    /// Generate node assignments for different component types
    fn generate_node_assignments(&self, component_type: &ComponentType) -> (String, String, String, String, String) {
        match component_type {
            ComponentType::Resistor | ComponentType::Capacitor | ComponentType::Inductor | 
            ComponentType::VoltageSource | ComponentType::CurrentSource | ComponentType::Diode => {
                ("1".to_string(), "0".to_string(), "2".to_string(), "3".to_string(), "4".to_string())
            },
            ComponentType::Transistor => {
                ("1".to_string(), "2".to_string(), "0".to_string(), "3".to_string(), "4".to_string())
            },
            ComponentType::OpAmp => {
                ("1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "0".to_string())
            },
        }
    }
    
    /// Get a unique component ID for the given type
    fn get_component_id(&mut self, component_type: &ComponentType) -> u32 {
        let counter = self.component_counter.entry(component_type.clone()).or_insert(0);
        *counter += 1;
        *counter
    }
    
    /// Parse a SPICE netlist into a circuit
    pub fn parse_netlist(&self, netlist: &str) -> Result<Circuit> {
        let mut circuit = Circuit::new();
        
        for line in netlist.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('*') || line.starts_with('.') {
                continue;
            }
            
            // Parse component line
            if let Ok(component) = self.parse_component_line(line) {
                circuit.components.push(component);
            }
        }
        
        Ok(circuit)
    }
    
    /// Parse a single component line
    fn parse_component_line(&self, line: &str) -> Result<Component> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        if parts.len() < 4 {
            return Err(SimulationError::ParseError {
                line: line.to_string(),
                reason: "Insufficient parameters".to_string(),
            });
        }
        
        let component_name = parts[0];
        let component_type = match component_name.chars().next().unwrap().to_ascii_uppercase() {
            'R' => ComponentType::Resistor,
            'C' => ComponentType::Capacitor,
            'L' => ComponentType::Inductor,
            'V' => ComponentType::VoltageSource,
            'I' => ComponentType::CurrentSource,
            'D' => ComponentType::Diode,
            'Q' => ComponentType::Transistor,
            'X' => ComponentType::OpAmp,
            _ => return Err(SimulationError::ParseError {
                line: line.to_string(),
                reason: "Unknown component type".to_string(),
            }),
        };
        
        let value = if parts.len() > 3 {
            Some(parts.last().unwrap().to_string())
        } else {
            None
        };
        
        Ok(Component {
            id: component_name.to_string(),
            component_type,
            value,
            position: (0.0, 0.0), // Default position
        })
    }
}

impl NetlistBuilder {
    /// Create a new netlist builder
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            components: Vec::new(),
            analysis_commands: Vec::new(),
            control_commands: Vec::new(),
        }
    }
    
    /// Add a component line
    pub fn add_component(&mut self, component: String) {
        self.components.push(component);
    }
    
    /// Add an analysis command
    pub fn add_analysis(&mut self, command: &str) {
        self.analysis_commands.push(command.to_string());
    }
    
    /// Add a control command
    pub fn add_control(&mut self, command: &str) {
        self.control_commands.push(command.to_string());
    }
    
    /// Build the complete netlist
    pub fn build(&self) -> String {
        let mut netlist = String::new();
        
        // Title line
        writeln!(netlist, "{}", self.title).unwrap();
        
        // Component lines
        for component in &self.components {
            writeln!(netlist, "{}", component).unwrap();
        }
        
        // Analysis commands
        for analysis in &self.analysis_commands {
            writeln!(netlist, "{}", analysis).unwrap();
        }
        
        // Control commands
        for control in &self.control_commands {
            writeln!(netlist, "{}", control).unwrap();
        }
        
        netlist
    }
}

impl Default for SpiceParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_resistor_circuit() {
        let mut parser = SpiceParser::new();
        
        let mut circuit = Circuit::new();
        circuit.add_component(Component {
            id: "R1".to_string(),
            component_type: ComponentType::Resistor,
            value: Some("1k".to_string()),
            position: (0.0, 0.0),
        });
        circuit.add_component(Component {
            id: "V1".to_string(),
            component_type: ComponentType::VoltageSource,
            value: Some("5".to_string()),
            position: (0.0, 0.0),
        });
        
        let netlist = parser.generate_netlist(&circuit).unwrap();
        println!("Generated netlist:\n{}", netlist);
        
        assert!(netlist.contains("R1"));
        assert!(netlist.contains("V1"));
        assert!(netlist.contains(".op"));
        assert!(netlist.contains(".end"));
    }
    
    #[test]
    fn test_parse_simple_netlist() {
        let parser = SpiceParser::new();
        
        let netlist = r#"
Simple test circuit
R1 1 0 1k
V1 1 0 DC 5
.op
.end
"#;
        
        let circuit = parser.parse_netlist(netlist).unwrap();
        assert_eq!(circuit.components.len(), 2);
        
        let resistor = &circuit.components[0];
        assert_eq!(resistor.component_type, ComponentType::Resistor);
        assert_eq!(resistor.value, Some("1k".to_string()));
        
        let voltage_source = &circuit.components[1];
        assert_eq!(voltage_source.component_type, ComponentType::VoltageSource);
        assert_eq!(voltage_source.value, Some("5".to_string()));
    }
}