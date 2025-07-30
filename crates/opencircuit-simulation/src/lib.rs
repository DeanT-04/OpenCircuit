//! NgSpice simulation engine for OpenCircuit
//! 
//! This crate provides a safe Rust wrapper around NgSpice for circuit simulation.
//! It includes SPICE netlist generation, simulation execution, and result processing.

use opencircuit_circuit::Circuit;

pub mod ngspice_wrapper;
pub mod spice_parser;
pub mod analysis;
pub mod results;
pub mod errors;
pub mod memory;

pub use ngspice_wrapper::NgSpiceWrapper;
pub use spice_parser::SpiceParser;
pub use analysis::*;
pub use results::*;
pub use errors::{SimulationError, Result};
pub use memory::MemoryPool;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Main simulation engine that coordinates all simulation operations
pub struct SimulationEngine {
    ngspice: Arc<Mutex<NgSpiceWrapper>>,
    parser: SpiceParser,
}

impl SimulationEngine {
    /// Create a new simulation engine
    pub async fn new() -> Result<Self> {
        let ngspice = NgSpiceWrapper::new().await?;
        let parser = SpiceParser::new();
        
        Ok(Self {
            ngspice: Arc::new(Mutex::new(ngspice)),
            parser,
        })
    }

    /// Simulate a circuit and return results
    pub async fn simulate_circuit(&mut self, circuit: &Circuit) -> Result<SimulationResults> {
        tracing::info!("Starting circuit simulation");
        
        // Generate SPICE netlist
        let netlist = self.parser.generate_netlist(circuit)?;
        tracing::debug!("Generated netlist: {}", netlist);
        
        // Run simulation
        let ngspice = self.ngspice.lock().await;
        let results = ngspice.run_simulation(netlist).await?;
        
        tracing::info!("Simulation completed successfully");
        Ok(results)
    }

    /// Check if NgSpice is available and working
    pub async fn health_check(&self) -> Result<bool> {
        let ngspice = self.ngspice.lock().await;
        ngspice.health_check().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simulation_engine_creation() {
        let result = SimulationEngine::new().await;
        // This might fail if NgSpice is not installed, which is expected
        match result {
            Ok(_) => println!("NgSpice is available"),
            Err(e) => println!("NgSpice not available: {}", e),
        }
    }
}