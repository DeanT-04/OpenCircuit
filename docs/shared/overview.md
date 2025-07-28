---
title: Shared Resources & Utilities
description: Common utilities, algorithms, and shared components across OpenCircuit
last_updated: 2025-01-27
tags: [shared, utilities, algorithms, common, tools]
context_id: shared.overview.main
---

# 🔧 Shared Resources & Utilities

This section contains common utilities, algorithms, and shared components used throughout OpenCircuit.

## 📚 Glossary

### Circuit Simulation Terms

**SPICE** - Simulation Program with Integrated Circuit Emphasis. Industry-standard circuit simulation format.

**Netlist** - Text description of electronic circuit connectivity and component values.

**Modified Nodal Analysis (MNA)** - Mathematical method for analyzing electronic circuits by formulating circuit equations.

**Operating Point** - DC steady-state solution of a circuit where all time derivatives are zero.

**Small Signal Analysis** - Linear approximation of circuit behavior around an operating point.

### PCB Design Terms

**Gerber Files** - Standard file format for PCB fabrication data including copper layers, solder mask, and drill files.

**Design Rules** - Constraints that define minimum spacing, trace widths, and other manufacturing requirements.

**Via** - Plated hole that connects traces on different layers of a PCB.

**Keepout Area** - Region where components or traces are not allowed to be placed.

**Copper Pour** - Large area of copper used for power/ground planes or EMI shielding.

### AI/ML Terms

**Embedding** - Dense vector representation of data that captures semantic meaning.

**Vector Database** - Database optimized for storing and searching high-dimensional vectors.

**Transformer** - Neural network architecture particularly effective for sequence-to-sequence tasks.

**Fine-tuning** - Process of adapting a pre-trained model to a specific task or domain.

**RAG (Retrieval-Augmented Generation)** - Technique combining information retrieval with language generation.

## 🧮 Common Algorithms

### Graph Algorithms
```rust
// @context_id: shared.algorithms.graph
// @purpose: Graph algorithms for circuit analysis
use std::collections::{HashMap, HashSet, VecDeque};

pub struct Graph<T> {
    nodes: HashMap<T, HashSet<T>>,
}

impl<T: Clone + Eq + std::hash::Hash> Graph<T> {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }
    
    pub fn add_edge(&mut self, from: T, to: T) {
        self.nodes.entry(from.clone()).or_default().insert(to.clone());
        self.nodes.entry(to).or_default().insert(from);
    }
    
    pub fn find_cycles(&self) -> Vec<Vec<T>> {
        let mut visited = HashSet::new();
        let mut cycles = Vec::new();
        
        for node in self.nodes.keys() {
            if !visited.contains(node) {
                self.dfs_cycles(node, &mut visited, &mut Vec::new(), &mut cycles);
            }
        }
        
        cycles
    }
    
    fn dfs_cycles(&self, node: &T, visited: &mut HashSet<T>, path: &mut Vec<T>, cycles: &mut Vec<Vec<T>>) {
        if path.contains(node) {
            // Found a cycle
            let cycle_start = path.iter().position(|n| n == node).unwrap();
            cycles.push(path[cycle_start..].to_vec());
            return;
        }
        
        if visited.contains(node) {
            return;
        }
        
        visited.insert(node.clone());
        path.push(node.clone());
        
        if let Some(neighbors) = self.nodes.get(node) {
            for neighbor in neighbors {
                self.dfs_cycles(neighbor, visited, path, cycles);
            }
        }
        
        path.pop();
    }
}
```

### Spatial Algorithms
```rust
// @context_id: shared.algorithms.spatial
// @purpose: Spatial data structures for component placement
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone)]
pub struct Rectangle {
    pub min: Point2D,
    pub max: Point2D,
}

impl Rectangle {
    pub fn contains(&self, point: &Point2D) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y
    }
    
    pub fn intersects(&self, other: &Rectangle) -> bool {
        !(self.max.x < other.min.x || other.max.x < self.min.x ||
          self.max.y < other.min.y || other.max.y < self.min.y)
    }
}

pub struct QuadTree<T> {
    boundary: Rectangle,
    capacity: usize,
    points: Vec<(Point2D, T)>,
    children: Option<Box<[QuadTree<T>; 4]>>,
}

impl<T> QuadTree<T> {
    pub fn new(boundary: Rectangle, capacity: usize) -> Self {
        Self {
            boundary,
            capacity,
            points: Vec::new(),
            children: None,
        }
    }
    
    pub fn insert(&mut self, point: Point2D, data: T) -> bool {
        if !self.boundary.contains(&point) {
            return false;
        }
        
        if self.points.len() < self.capacity && self.children.is_none() {
            self.points.push((point, data));
            return true;
        }
        
        if self.children.is_none() {
            self.subdivide();
        }
        
        if let Some(ref mut children) = self.children {
            for child in children.iter_mut() {
                if child.insert(point, data) {
                    return true;
                }
            }
        }
        
        false
    }
    
    fn subdivide(&mut self) {
        let mid_x = (self.boundary.min.x + self.boundary.max.x) / 2.0;
        let mid_y = (self.boundary.min.y + self.boundary.max.y) / 2.0;
        
        let nw = Rectangle {
            min: self.boundary.min,
            max: Point2D { x: mid_x, y: mid_y },
        };
        let ne = Rectangle {
            min: Point2D { x: mid_x, y: self.boundary.min.y },
            max: Point2D { x: self.boundary.max.x, y: mid_y },
        };
        let sw = Rectangle {
            min: Point2D { x: self.boundary.min.x, y: mid_y },
            max: Point2D { x: mid_x, y: self.boundary.max.y },
        };
        let se = Rectangle {
            min: Point2D { x: mid_x, y: mid_y },
            max: self.boundary.max,
        };
        
        self.children = Some(Box::new([
            QuadTree::new(nw, self.capacity),
            QuadTree::new(ne, self.capacity),
            QuadTree::new(sw, self.capacity),
            QuadTree::new(se, self.capacity),
        ]));
    }
}
```

## 🔧 Utility Functions

### File I/O Utilities
```rust
// @context_id: shared.utils.file_io
// @purpose: Common file operations for OpenCircuit
use std::path::Path;
use tokio::fs;
use serde::{Serialize, Deserialize};

pub async fn save_json<T: Serialize>(data: &T, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(data)?;
    fs::write(path, json).await?;
    Ok(())
}

pub async fn load_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path).await?;
    let data = serde_json::from_str(&content)?;
    Ok(data)
}

pub async fn backup_file(original: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if original.exists() {
        let backup_name = format!("{}.backup", original.display());
        fs::copy(original, &backup_name).await?;
    }
    Ok(())
}
```

### Math Utilities
```rust
// @context_id: shared.utils.math
// @purpose: Mathematical utilities for circuit calculations
use std::f64::consts::PI;

pub fn db_to_linear(db: f64) -> f64 {
    10.0_f64.powf(db / 20.0)
}

pub fn linear_to_db(linear: f64) -> f64 {
    20.0 * linear.log10()
}

pub fn parallel_resistance(r1: f64, r2: f64) -> f64 {
    (r1 * r2) / (r1 + r2)
}

pub fn series_resistance(resistors: &[f64]) -> f64 {
    resistors.iter().sum()
}

pub fn parallel_impedance(impedances: &[Complex<f64>]) -> Complex<f64> {
    let sum_reciprocals = impedances.iter()
        .map(|z| 1.0 / z)
        .sum::<Complex<f64>>();
    1.0 / sum_reciprocals
}

pub fn frequency_to_omega(frequency: f64) -> f64 {
    2.0 * PI * frequency
}

pub fn omega_to_frequency(omega: f64) -> f64 {
    omega / (2.0 * PI)
}
```

### Validation Utilities
```rust
// @context_id: shared.utils.validation
// @purpose: Input validation and sanitization
use regex::Regex;

pub struct Validator;

impl Validator {
    pub fn is_valid_component_name(name: &str) -> bool {
        let re = Regex::new(r"^[A-Za-z][A-Za-z0-9_]*$").unwrap();
        re.is_match(name) && name.len() <= 64
    }
    
    pub fn is_valid_net_name(name: &str) -> bool {
        let re = Regex::new(r"^[A-Za-z0-9_/]+$").unwrap();
        re.is_match(name) && name.len() <= 128
    }
    
    pub fn sanitize_filename(filename: &str) -> String {
        let re = Regex::new(r"[<>:\"/\\|?*]").unwrap();
        re.replace_all(filename, "_").to_string()
    }
    
    pub fn validate_frequency_range(start: f64, stop: f64) -> Result<(), String> {
        if start <= 0.0 || stop <= 0.0 {
            return Err("Frequencies must be positive".to_string());
        }
        if start >= stop {
            return Err("Start frequency must be less than stop frequency".to_string());
        }
        Ok(())
    }
}
```

## 🔗 Cross-Module Integration

### Event System
```rust
// @context_id: shared.events.system
// @purpose: Event-driven communication between modules
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum OpenCircuitEvent {
    CircuitLoaded { path: String },
    ComponentAdded { component_id: String },
    SimulationStarted { analysis_type: String },
    SimulationCompleted { results: SimulationResult },
    PCBLayoutChanged,
    AIAssistantQuery { query: String },
}

pub struct EventBus {
    sender: broadcast::Sender<OpenCircuitEvent>,
    subscribers: Arc<Mutex<HashMap<String, broadcast::Receiver<OpenCircuitEvent>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self {
            sender,
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn publish(&self, event: OpenCircuitEvent) {
        let _ = self.sender.send(event);
    }
    
    pub fn subscribe(&self, subscriber_id: String) -> broadcast::Receiver<OpenCircuitEvent> {
        let receiver = self.sender.subscribe();
        self.subscribers.lock().unwrap().insert(subscriber_id, receiver.clone());
        receiver
    }
}
```

## 📊 Performance Monitoring
```rust
// @context_id: shared.performance.monitoring
// @purpose: Performance tracking and profiling utilities
use std::time::{Duration, Instant};
use std::collections::HashMap;

pub struct PerformanceMonitor {
    timers: HashMap<String, Instant>,
    measurements: HashMap<String, Vec<Duration>>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            timers: HashMap::new(),
            measurements: HashMap::new(),
        }
    }
    
    pub fn start_timer(&mut self, name: &str) {
        self.timers.insert(name.to_string(), Instant::now());
    }
    
    pub fn end_timer(&mut self, name: &str) -> Option<Duration> {
        if let Some(start_time) = self.timers.remove(name) {
            let duration = start_time.elapsed();
            self.measurements.entry(name.to_string())
                .or_default()
                .push(duration);
            Some(duration)
        } else {
            None
        }
    }
    
    pub fn get_average_time(&self, name: &str) -> Option<Duration> {
        if let Some(measurements) = self.measurements.get(name) {
            if measurements.is_empty() {
                return None;
            }
            let total: Duration = measurements.iter().sum();
            Some(total / measurements.len() as u32)
        } else {
            None
        }
    }
}
```

---

*Context ID: shared.overview.main*