//! Circuit validation and verification utilities
//! Provides comprehensive checking for circuit correctness and design rules

use super::netlist::{ComponentType, Netlist};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Circuit validation error: {0}")]
    ValidationError(String),
    #[error("Missing ground reference: {0}")]
    MissingGround(String),
    #[error("Unconnected node: {0}")]
    UnconnectedNode(String),
    #[error("Invalid component value: {0}")]
    InvalidValue(String),
    #[error("Short circuit detected: {0}")]
    ShortCircuit(String),
    #[error("Missing analysis type: {0}")]
    MissingAnalysis(String),
    #[error("Component naming conflict: {0}")]
    NamingConflict(String),
    #[error("Floating node detected: {0}")]
    FloatingNode(String),
    #[error("Invalid node connection: {0}")]
    InvalidConnection(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
    pub metrics: ValidationMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetrics {
    pub component_count: usize,
    pub node_count: usize,
    pub voltage_sources: usize,
    pub current_sources: usize,
    pub resistors: usize,
    pub capacitors: usize,
    pub inductors: usize,
    pub diodes: usize,
    pub transistors: usize,
}

#[derive(Debug, Clone)]
pub struct DesignRule {
    pub name: String,
    pub description: String,
    pub check_function: String, // Would be function pointer in real implementation
    pub severity: RuleSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleSeverity {
    Error,
    Warning,
    Info,
}

pub struct CircuitValidator {
    design_rules: Vec<DesignRule>,
    min_component_values: HashMap<ComponentType, f64>,
    max_component_values: HashMap<ComponentType, f64>,
}

impl CircuitValidator {
    pub fn new() -> Self {
        let mut validator = Self {
            design_rules: Vec::new(),
            min_component_values: HashMap::new(),
            max_component_values: HashMap::new(),
        };

        validator.initialize_default_rules();
        validator
    }

    fn initialize_default_rules(&mut self) {
        // Add basic design rules
        self.design_rules.push(DesignRule {
            name: "Ground Reference".to_string(),
            description: "Circuit must have a ground reference (node 0)".to_string(),
            check_function: "check_ground_reference".to_string(),
            severity: RuleSeverity::Error,
        });

        self.design_rules.push(DesignRule {
            name: "No Floating Nodes".to_string(),
            description: "All nodes should be connected to at least two components".to_string(),
            check_function: "check_floating_nodes".to_string(),
            severity: RuleSeverity::Warning,
        });

        self.design_rules.push(DesignRule {
            name: "Component Values".to_string(),
            description: "Component values should be within reasonable ranges".to_string(),
            check_function: "check_component_values".to_string(),
            severity: RuleSeverity::Warning,
        });

        self.design_rules.push(DesignRule {
            name: "Short Circuit".to_string(),
            description: "Check for direct shorts between voltage sources".to_string(),
            check_function: "check_short_circuits".to_string(),
            severity: RuleSeverity::Error,
        });

        // Set reasonable component value ranges
        self.min_component_values.insert(ComponentType::Resistor, 1e-3); // 1 mΩ
        self.max_component_values.insert(ComponentType::Resistor, 1e9); // 1 GΩ

        self.min_component_values.insert(ComponentType::Capacitor, 1e-15); // 1 fF
        self.max_component_values.insert(ComponentType::Capacitor, 100.0); // 100 F

        self.min_component_values.insert(ComponentType::Inductor, 1e-12); // 1 pH
        self.max_component_values.insert(ComponentType::Inductor, 1000.0); // 1000 H
    }

    pub fn validate(&self, netlist: &Netlist) -> ValidationReport {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();

        // Collect metrics
        let metrics = self.calculate_metrics(netlist);

        // Perform validation checks
        if let Err(e) = self.check_ground_reference(netlist) {
            errors.push(e.to_string());
        }

        if let Err(e) = self.check_floating_nodes(netlist) {
            warnings.push(e.to_string());
        }

        if let Err(e) = self.check_component_values(netlist) {
            warnings.push(e.to_string());
        }

        if let Err(e) = self.check_short_circuits(netlist) {
            errors.push(e.to_string());
        }

        if let Err(e) = self.check_naming_conflicts(netlist) {
            errors.push(e.to_string());
        }

        // Add recommendations
        self.add_recommendations(netlist, &mut recommendations);

        ValidationReport {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            recommendations,
            metrics,
        }
    }

    fn calculate_metrics(&self, netlist: &Netlist) -> ValidationMetrics {
        let mut metrics = ValidationMetrics {
            component_count: netlist.components.len(),
            node_count: 0,
            voltage_sources: 0,
            current_sources: 0,
            resistors: 0,
            capacitors: 0,
            inductors: 0,
            diodes: 0,
            transistors: 0,
        };

        let mut unique_nodes = HashSet::new();

        for component in &netlist.components {
            for node in &component.nodes {
                unique_nodes.insert(node.clone());
            }

            match component.component_type {
                ComponentType::VoltageSource => metrics.voltage_sources += 1,
                ComponentType::CurrentSource => metrics.current_sources += 1,
                ComponentType::Resistor => metrics.resistors += 1,
                ComponentType::Capacitor => metrics.capacitors += 1,
                ComponentType::Inductor => metrics.inductors += 1,
                ComponentType::Diode => metrics.diodes += 1,
                ComponentType::Bjt | ComponentType::Mosfet => metrics.transistors += 1,
                _ => {}
            }
        }

        metrics.node_count = unique_nodes.len();
        metrics
    }

    fn check_ground_reference(&self, netlist: &Netlist) -> Result<(), ValidationError> {
        let mut has_ground = false;

        for component in &netlist.components {
            if component.nodes.contains(&"0".to_string()) {
                has_ground = true;
                break;
            }
        }

        if has_ground {
            Ok(())
        } else {
            Err(ValidationError::MissingGround(
                "Circuit must have at least one connection to ground (node 0)".to_string(),
            ))
        }
    }

    fn check_floating_nodes(&self, netlist: &Netlist) -> Result<(), ValidationError> {
        let mut node_connections: HashMap<String, usize> = HashMap::new();

        for component in &netlist.components {
            for node in &component.nodes {
                *node_connections.entry(node.clone()).or_insert(0) += 1;
            }
        }

        let mut floating_nodes = Vec::new();
        for (node, connections) in &node_connections {
            if *connections < 2 && node != "0" {
                floating_nodes.push(node.clone());
            }
        }

        if floating_nodes.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::FloatingNode(format!(
                "Floating nodes detected: {}",
                floating_nodes.join(", ")
            )))
        }
    }

    fn check_component_values(&self, netlist: &Netlist) -> Result<(), ValidationError> {
        let mut invalid_values = Vec::new();

        for component in &netlist.components {
            if let Ok(value) = self.parse_component_value(&component.value) {
                if let Some(min_val) = self.min_component_values.get(&component.component_type) {
                    if value < *min_val {
                        invalid_values.push(format!(
                            "{}: value {} below minimum {}",
                            component.name, component.value, min_val
                        ));
                    }
                }

                if let Some(max_val) = self.max_component_values.get(&component.component_type) {
                    if value > *max_val {
                        invalid_values.push(format!(
                            "{}: value {} above maximum {}",
                            component.name, component.value, max_val
                        ));
                    }
                }
            } else {
                invalid_values.push(format!(
                    "{}: unable to parse value '{}'",
                    component.name, component.value
                ));
            }
        }

        if invalid_values.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::InvalidValue(invalid_values.join("; ")))
        }
    }

    fn check_short_circuits(&self, netlist: &Netlist) -> Result<(), ValidationError> {
        let mut voltage_sources = Vec::new();

        for component in &netlist.components {
            if matches!(component.component_type, ComponentType::VoltageSource) {
                voltage_sources.push(component);
            }
        }

        // Check for direct shorts between voltage sources
        for source1 in &voltage_sources {
            for source2 in &voltage_sources {
                if source1.name != source2.name {
                    let common_nodes: Vec<&String> = source1.nodes
                        .iter()
                        .filter(|node| source2.nodes.contains(node))
                        .collect();
                    
                    if common_nodes.len() >= 2 {
                        return Err(ValidationError::ShortCircuit(format!(
                            "Potential short circuit between {} and {}",
                            source1.name, source2.name
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    fn check_naming_conflicts(&self, netlist: &Netlist) -> Result<(), ValidationError> {
        let mut names = HashSet::new();
        let mut conflicts = Vec::new();

        for component in &netlist.components {
            if !names.insert(&component.name) {
                conflicts.push(component.name.clone());
            }
        }

        if conflicts.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::NamingConflict(format!(
                "Duplicate component names: {}",
                conflicts.join(", ")
            )))
        }
    }

    fn add_recommendations(&self, netlist: &Netlist, recommendations: &mut Vec<String>) {
        let metrics = self.calculate_metrics(netlist);

        if metrics.voltage_sources == 0 {
            recommendations.push("Consider adding a voltage source to power the circuit".to_string());
        }

        if metrics.resistors == 0 && metrics.capacitors == 0 && metrics.inductors == 0 {
            recommendations.push("Circuit appears to have no passive components".to_string());
        }

        if metrics.node_count > 20 {
            recommendations.push("Large circuit - consider breaking into sub-circuits".to_string());
        }

        if metrics.voltage_sources > 5 {
            recommendations.push("Many voltage sources - consider using voltage dividers".to_string());
        }
    }

    fn parse_component_value(&self, value_str: &str) -> Result<f64, ()> {
        let value_str = value_str.to_lowercase();
        
        // Remove common suffixes and prefixes
        let cleaned = value_str
            .replace("ohm", "")
            .replace("ω", "")
            .replace("h", "")
            .replace("f", "")
            .replace("v", "")
            .replace("a", "")
            .replace("hz", "");

        let multiplier = if cleaned.ends_with('k') {
            1e3
        } else if cleaned.ends_with('m') {
            1e-3
        } else if cleaned.ends_with('u') || cleaned.ends_with('μ') {
            1e-6
        } else if cleaned.ends_with('n') {
            1e-9
        } else if cleaned.ends_with('p') {
            1e-12
        } else if cleaned.ends_with('g') {
            1e9
        } else {
            1.0
        };

        let numeric_str = cleaned.trim_end_matches(|c: char| c.is_alphabetic());
        
        numeric_str.parse::<f64>().map(|v| v * multiplier).map_err(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::netlist::{Component, ComponentType, Netlist};

    #[test]
    fn test_validation_with_valid_circuit() {
        let mut netlist = Netlist::new("Test Circuit".to_string());
        
        netlist.components.push(Component {
            name: "V1".to_string(),
            component_type: ComponentType::VoltageSource,
            nodes: vec!["1".to_string(), "0".to_string()],
            value: "12".to_string(),
            model: None,
            parameters: HashMap::new(),
        });

        netlist.components.push(Component {
            name: "R1".to_string(),
            component_type: ComponentType::Resistor,
            nodes: vec!["1".to_string(), "2".to_string()],
            value: "1k".to_string(),
            model: None,
            parameters: HashMap::new(),
        });

        netlist.components.push(Component {
            name: "R2".to_string(),
            component_type: ComponentType::Resistor,
            nodes: vec!["2".to_string(), "0".to_string()],
            value: "1k".to_string(),
            model: None,
            parameters: HashMap::new(),
        });

        let validator = CircuitValidator::new();
        let report = validator.validate(&netlist);

        assert!(report.is_valid);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn test_validation_missing_ground() {
        let mut netlist = Netlist::new("Test Circuit".to_string());
        
        netlist.components.push(Component {
            name: "R1".to_string(),
            component_type: ComponentType::Resistor,
            nodes: vec!["1".to_string(), "2".to_string()],
            value: "1k".to_string(),
            model: None,
            parameters: HashMap::new(),
        });

        let validator = CircuitValidator::new();
        let report = validator.validate(&netlist);

        assert!(!report.is_valid);
        assert!(!report.errors.is_empty());
    }

    #[test]
    fn test_parse_component_value() {
        let validator = CircuitValidator::new();

        assert_eq!(validator.parse_component_value("1k"), Ok(1000.0));
        assert_eq!(validator.parse_component_value("1.5M"), Ok(1_500_000.0));
        assert_eq!(validator.parse_component_value("10u"), Ok(10e-6));
        assert_eq!(validator.parse_component_value("100n"), Ok(100e-9));
    }

    #[test]
    fn test_metrics_calculation() {
        let mut netlist = Netlist::new("Test Circuit".to_string());
        
        netlist.components.push(Component {
            name: "V1".to_string(),
            component_type: ComponentType::VoltageSource,
            nodes: vec!["1".to_string(), "0".to_string()],
            value: "12".to_string(),
            model: None,
            parameters: HashMap::new(),
        });

        netlist.components.push(Component {
            name: "R1".to_string(),
            component_type: ComponentType::Resistor,
            nodes: vec!["1".to_string(), "2".to_string()],
            value: "1k".to_string(),
            model: None,
            parameters: HashMap::new(),
        });

        let validator = CircuitValidator::new();
        let metrics = validator.calculate_metrics(&netlist);

        assert_eq!(metrics.component_count, 2);
        assert_eq!(metrics.voltage_sources, 1);
        assert_eq!(metrics.resistors, 1);
    }
}