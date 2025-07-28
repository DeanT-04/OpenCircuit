---
title: Circuit Simulation Overview
description: Comprehensive guide to circuit simulation capabilities in OpenCircuit
last_updated: 2025-01-27
tags: [circuit, simulation, ngspice, spice, analysis]
context_id: circuit.overview.main
---

# ⚡ Circuit Simulation Overview

OpenCircuit provides advanced circuit simulation capabilities through NgSpice integration and native Rust implementations.

## 🔬 Simulation Engine Architecture

```
┌─────────────────────────────────────┐
│        OpenCircuit Frontend        │
├─────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐   │
│  │   Native    │  │   NgSpice   │   │
│  │ Rust Solver │  │  Integration│   │
│  └─────────────┘  └─────────────┘   │
├─────────────────────────────────────┤
│         Circuit Netlist            │
│       (SPICE Compatible)           │
├─────────────────────────────────────┤
│        Analysis Engines            │
│   DC • AC • Transient • Noise     │
└─────────────────────────────────────┘
```

## 🧮 Simulation Types

### DC Analysis
- **Operating Point** - Find steady-state voltages and currents
- **DC Sweep** - Vary DC sources and plot results
- **Transfer Function** - Small-signal gain and impedance

```rust
// @context_id: circuit.simulation.dc_analysis
// @purpose: Configure DC analysis parameters
use crate::simulation::{AnalysisType, DCAnalysis};

let dc_analysis = DCAnalysis {
    analysis_type: AnalysisType::OperatingPoint,
    sweep_source: Some("VIN".to_string()),
    start_value: 0.0,
    stop_value: 5.0,
    step_size: 0.1,
    temperature: 27.0, // Celsius
};
```

### AC Analysis
- **Frequency Response** - Magnitude and phase vs frequency
- **Bode Plots** - Gain and phase margins
- **Noise Analysis** - Input and output noise

```rust
// @context_id: circuit.simulation.ac_analysis
// @purpose: Configure AC frequency sweep
let ac_analysis = ACAnalysis {
    analysis_type: AnalysisType::FrequencyResponse,
    start_frequency: 1.0,      // Hz
    stop_frequency: 1e6,       // Hz
    points_per_decade: 100,
    sweep_type: SweepType::Logarithmic,
};
```

### Transient Analysis
- **Time Domain** - Voltage and current vs time
- **Pulse Response** - Step and impulse responses
- **Oscillation** - Periodic steady-state

```rust
// @context_id: circuit.simulation.transient_analysis
// @purpose: Configure transient time analysis
let transient_analysis = TransientAnalysis {
    start_time: 0.0,
    stop_time: 1e-3,           // 1ms
    time_step: 1e-6,           // 1µs
    max_time_step: 1e-5,       // 10µs
    initial_conditions: HashMap::new(),
};
```

## 🔧 NgSpice Integration

### Features
- **Industry Standard** - SPICE 3f5 compatible
- **Comprehensive Models** - Extensive device library
- **Advanced Analysis** - Monte Carlo, sensitivity
- **Convergence Control** - Robust numerical methods

### Rust Bindings
```rust
// @context_id: circuit.ngspice.integration
// @purpose: NgSpice wrapper for circuit simulation
use paprika::{NgSpice, SpiceResult};

pub struct NgSpiceSimulator {
    engine: NgSpice,
    netlist: String,
}

impl NgSpiceSimulator {
    pub fn new() -> Result<Self, SimulationError> {
        let engine = NgSpice::new()?;
        Ok(Self {
            engine,
            netlist: String::new(),
        })
    }
    
    pub fn load_netlist(&mut self, netlist: &str) -> Result<(), SimulationError> {
        self.netlist = netlist.to_string();
        self.engine.circuit_from_string(&self.netlist)?;
        Ok(())
    }
    
    pub fn run_analysis(&mut self, analysis: &str) -> Result<SimulationResult, SimulationError> {
        let command = format!("run {}", analysis);
        self.engine.command(&command)?;
        
        // Extract results
        let voltages = self.engine.get_vector_data("v")?;
        let currents = self.engine.get_vector_data("i")?;
        
        Ok(SimulationResult {
            voltages,
            currents,
            time: self.engine.get_vector_data("time").ok(),
            frequency: self.engine.get_vector_data("frequency").ok(),
        })
    }
}
```

## 🦀 Native Rust Simulation

### Pure Rust Solvers
- **spice-oxide** - Native SPICE implementation
- **electrical** - Circuit analysis library
- **Custom solvers** - Optimized for specific use cases

```rust
// @context_id: circuit.rust.native_solver
// @purpose: Native Rust circuit solver implementation
use nalgebra::{DMatrix, DVector};

pub struct NativeCircuitSolver {
    conductance_matrix: DMatrix<f64>,
    current_vector: DVector<f64>,
    node_count: usize,
}

impl NativeCircuitSolver {
    pub fn solve_dc(&self) -> Result<DVector<f64>, SolverError> {
        // Modified Nodal Analysis (MNA)
        let lu = self.conductance_matrix.lu();
        match lu.solve(&self.current_vector) {
            Some(solution) => Ok(solution),
            None => Err(SolverError::SingularMatrix),
        }
    }
    
    pub fn solve_ac(&self, frequency: f64) -> Result<Vec<Complex<f64>>, SolverError> {
        // AC analysis with complex arithmetic
        let omega = 2.0 * std::f64::consts::PI * frequency;
        let impedance_matrix = self.build_impedance_matrix(omega);
        
        // Solve complex linear system
        self.solve_complex_system(&impedance_matrix)
    }
}
```

## 📊 Analysis Results

### Data Structures
```rust
// @context_id: circuit.simulation.results
// @purpose: Define simulation result data structures
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub analysis_type: AnalysisType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub node_voltages: HashMap<String, Vec<f64>>,
    pub branch_currents: HashMap<String, Vec<f64>>,
    pub sweep_variable: Option<Vec<f64>>,
    pub convergence_info: ConvergenceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceInfo {
    pub iterations: usize,
    pub final_error: f64,
    pub converged: bool,
    pub warnings: Vec<String>,
}
```

### Visualization Integration
```rust
// @context_id: circuit.simulation.plotting
// @purpose: Plot simulation results with egui_plot
use egui_plot::{Line, Plot, PlotPoints};

impl SimulationResult {
    pub fn plot_voltage(&self, node: &str, ui: &mut egui::Ui) {
        if let Some(voltages) = self.node_voltages.get(node) {
            let points: PlotPoints = self.sweep_variable
                .as_ref()
                .unwrap_or(&(0..voltages.len()).map(|i| i as f64).collect())
                .iter()
                .zip(voltages.iter())
                .map(|(&x, &y)| [x, y])
                .collect();
            
            Plot::new(format!("voltage_{}", node))
                .view_aspect(2.0)
                .show(ui, |plot_ui| {
                    plot_ui.line(Line::new(points).name(format!("V({})", node)));
                });
        }
    }
}
```

## 🎯 Performance Optimization

### Parallel Processing
- **Multi-threaded analysis** - Parallel frequency sweeps
- **GPU acceleration** - Matrix operations on GPU
- **Caching** - Reuse factorizations

### Memory Management
- **Sparse matrices** - Efficient storage for large circuits
- **Streaming results** - Handle large datasets
- **Memory pools** - Reduce allocation overhead

## 🔗 Quick Links

- [NgSpice Integration Guide](ngspice.md)
- [Rust Circuit Crates](rust_crates.md)
- [Circuit Analysis Theory](theory.md)
- [Performance Tuning](performance.md)

---

*Context ID: circuit.overview.main*