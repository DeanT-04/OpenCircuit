//! Animation system for circuit simulation visualization
//! 
//! Provides smooth animations for real-time circuit simulation results,
//! including current flow, voltage levels, and interactive effects.

use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Animation manager for circuit simulation
pub struct CircuitAnimations {
    /// Current animation state
    animations: HashMap<String, Animation>,
    /// Global animation speed multiplier
    speed: f32,
    /// Animation time accumulator
    time: f32,
    /// Last update time
    last_update: Instant,
}

impl CircuitAnimations {
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
            speed: 1.0,
            time: 0.0,
            last_update: Instant::now(),
        }
    }

    /// Update all animations
    pub fn update(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_update);
        self.last_update = now;
        
        self.time += delta.as_secs_f32() * self.speed;
        
        // Remove completed animations
        self.animations.retain(|_, anim| !anim.is_complete());
        
        // Update active animations
        for anim in self.animations.values_mut() {
            anim.update(delta);
        }
    }

    /// Add current flow animation along a wire
    pub fn add_current_flow(&mut self, wire_id: String, current: f64, duration: Duration) {
        let anim = Animation::CurrentFlow(CurrentFlowAnimation {
            wire_id,
            current: current.abs(),
            direction: if current >= 0.0 { 1.0 } else { -1.0 },
            duration,
            elapsed: Duration::ZERO,
            particles: Vec::new(),
        });
        
        self.animations.insert(format!("current_{}", wire_id), anim);
    }

    /// Add voltage level animation for a component
    pub fn add_voltage_level(&mut self, component_id: String, voltage: f64, duration: Duration) {
        let anim = Animation::VoltageLevel(VoltageLevelAnimation {
            component_id,
            voltage,
            duration,
            elapsed: Duration::ZERO,
            pulse_intensity: 0.0,
        });
        
        self.animations.insert(format!("voltage_{}", component_id), anim);
    }

    /// Add selection highlight animation
    pub fn add_selection_highlight(&mut self, component_id: String, duration: Duration) {
        let anim = Animation::SelectionHighlight(SelectionHighlightAnimation {
            component_id,
            duration,
            elapsed: Duration::ZERO,
            pulse_phase: 0.0,
        });
        
        self.animations.insert(format!("select_{}", component_id), anim);
    }

    /// Add connection animation for new components
    pub fn add_connection(&mut self, from_pos: Pos2, to_pos: Pos2, duration: Duration) {
        let anim = Animation::Connection(ConnectionAnimation {
            from_pos,
            to_pos,
            duration,
            elapsed: Duration::ZERO,
            progress: 0.0,
        });
        
        self.animations.insert(format!("conn_{}_{}", from_pos.x, from_pos.y), anim);
    }

    /// Add simulation running indicator
    pub fn add_simulation_indicator(&mut self, center: Pos2, duration: Duration) {
        let anim = Animation::SimulationIndicator(SimulationIndicatorAnimation {
            center,
            duration,
            elapsed: Duration::ZERO,
            rotation: 0.0,
        });
        
        self.animations.insert("sim_indicator".to_string(), anim);
    }

    /// Render all animations
    pub fn render(&self, ui: &mut Ui) {
        for anim in self.animations.values() {
            anim.render(ui);
        }
    }

    /// Get animation progress for a specific animation
    pub fn get_progress(&self, id: &str) -> Option<f32> {
        self.animations.get(id).map(|anim| anim.progress())
    }

    /// Set global animation speed
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.max(0.1).min(5.0);
    }

    /// Clear all animations
    pub fn clear(&mut self) {
        self.animations.clear();
    }

    /// Remove specific animation
    pub fn remove(&mut self, id: &str) {
        self.animations.remove(id);
    }
}

/// Animation types
#[derive(Debug, Clone)]
enum Animation {
    CurrentFlow(CurrentFlowAnimation),
    VoltageLevel(VoltageLevelAnimation),
    SelectionHighlight(SelectionHighlightAnimation),
    Connection(ConnectionAnimation),
    SimulationIndicator(SimulationIndicatorAnimation),
}

impl Animation {
    fn update(&mut self, delta: Duration) {
        match self {
            Animation::CurrentFlow(anim) => anim.update(delta),
            Animation::VoltageLevel(anim) => anim.update(delta),
            Animation::SelectionHighlight(anim) => anim.update(delta),
            Animation::Connection(anim) => anim.update(delta),
            Animation::SimulationIndicator(anim) => anim.update(delta),
        }
    }

    fn render(&self, ui: &mut Ui) {
        match self {
            Animation::CurrentFlow(anim) => anim.render(ui),
            Animation::VoltageLevel(anim) => anim.render(ui),
            Animation::SelectionHighlight(anim) => anim.render(ui),
            Animation::Connection(anim) => anim.render(ui),
            Animation::SimulationIndicator(anim) => anim.render(ui),
        }
    }

    fn is_complete(&self) -> bool {
        match self {
            Animation::CurrentFlow(anim) => anim.is_complete(),
            Animation::VoltageLevel(anim) => anim.is_complete(),
            Animation::SelectionHighlight(anim) => anim.is_complete(),
            Animation::Connection(anim) => anim.is_complete(),
            Animation::SimulationIndicator(anim) => anim.is_complete(),
        }
    }

    fn progress(&self) -> f32 {
        match self {
            Animation::CurrentFlow(anim) => anim.progress(),
            Animation::VoltageLevel(anim) => anim.progress(),
            Animation::SelectionHighlight(anim) => anim.progress(),
            Animation::Connection(anim) => anim.progress(),
            Animation::SimulationIndicator(anim) => anim.progress(),
        }
    }
}

/// Current flow animation along wires
#[derive(Debug, Clone)]
struct CurrentFlowAnimation {
    wire_id: String,
    current: f64,
    direction: f32,
    duration: Duration,
    elapsed: Duration,
    particles: Vec<Particle>,
}

impl CurrentFlowAnimation {
    fn update(&mut self, delta: Duration) {
        self.elapsed += delta;
        
        // Spawn new particles
        if self.elapsed.as_secs_f32() % 0.1 < delta.as_secs_f32() {
            self.particles.push(Particle {
                position: 0.0,
                speed: 0.5 + self.current as f32 * 0.1,
                size: 3.0,
                color: self.get_current_color(),
            });
        }
        
        // Update particles
        for particle in &mut self.particles {
            particle.position += particle.speed * self.direction;
        }
        
        // Remove particles that have moved past the wire
        self.particles.retain(|p| p.position >= 0.0 && p.position <= 1.0);
    }

    fn render(&self, ui: &mut Ui) {
        let painter = ui.painter();
        
        for particle in &self.particles {
            // This would be positioned along the actual wire path
            // For now, we'll use a placeholder position
            let pos = Pos2::new(100.0, 100.0);
            painter.circle(pos, particle.size, particle.color, Stroke::NONE);
        }
    }

    fn is_complete(&self) -> bool {
        self.elapsed >= self.duration && self.particles.is_empty()
    }

    fn progress(&self) -> f32 {
        (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0)
    }

    fn get_current_color(&self) -> Color32 {
        let intensity = (self.current / 1.0).min(1.0);
        let red = (intensity * 255.0) as u8;
        let blue = ((1.0 - intensity) * 255.0) as u8;
        Color32::from_rgb(red, 0, blue)
    }
}

/// Voltage level indication animation
#[derive(Debug, Clone)]
struct VoltageLevelAnimation {
    component_id: String,
    voltage: f64,
    duration: Duration,
    elapsed: Duration,
    pulse_intensity: f32,
}

impl VoltageLevelAnimation {
    fn update(&mut self, delta: Duration) {
        self.elapsed += delta;
        
        // Calculate pulse intensity based on voltage level
        let normalized_voltage = (self.voltage / 10.0).min(1.0);
        let pulse_freq = 2.0; // Hz
        let phase = self.elapsed.as_secs_f32() * pulse_freq * 2.0 * std::f32::consts::PI;
        self.pulse_intensity = normalized_voltage as f32 * (1.0 + 0.3 * phase.sin());
    }

    fn render(&self, ui: &mut Ui) {
        let painter = ui.painter();
        
        // This would be positioned at the actual component
        // For now, we'll use a placeholder position
        let pos = Pos2::new(200.0, 100.0);
        let radius = 20.0 + self.pulse_intensity * 10.0;
        let color = self.get_voltage_color();
        
        painter.circle(pos, radius, color, Stroke::NONE);
    }

    fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }

    fn progress(&self) -> f32 {
        (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0)
    }

    fn get_voltage_color(&self) -> Color32 {
        let normalized = (self.voltage / 10.0).min(1.0);
        let red = (normalized * 255.0) as u8;
        let green = ((1.0 - normalized) * 255.0) as u8;
        Color32::from_rgba_premultiplied(red, green, 0, (normalized * 128.0) as u8)
    }
}

/// Selection highlight animation
#[derive(Debug, Clone)]
struct SelectionHighlightAnimation {
    component_id: String,
    duration: Duration,
    elapsed: Duration,
    pulse_phase: f32,
}

impl SelectionHighlightAnimation {
    fn update(&mut self, delta: Duration) {
        self.elapsed += delta;
        let pulse_freq = 3.0; // Hz
        let phase = self.elapsed.as_secs_f32() * pulse_freq * 2.0 * std::f32::consts::PI;
        self.pulse_phase = (1.0 + phase.sin()) * 0.5;
    }

    fn render(&self, ui: &mut Ui) {
        let painter = ui.painter();
        
        // This would be positioned at the actual component
        // For now, we'll use a placeholder position
        let pos = Pos2::new(300.0, 100.0);
        let radius = 25.0 + self.pulse_phase * 5.0;
        let alpha = (self.pulse_phase * 128.0) as u8;
        
        painter.circle_stroke(pos, radius, Stroke::new(2.0, Color32::from_rgba_premultiplied(255, 165, 0, alpha)));
    }

    fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }

    fn progress(&self) -> f32 {
        (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0)
    }
}

/// Connection animation for new components
#[derive(Debug, Clone)]
struct ConnectionAnimation {
    from_pos: Pos2,
    to_pos: Pos2,
    duration: Duration,
    elapsed: Duration,
    progress: f32,
}

impl ConnectionAnimation {
    fn update(&mut self, delta: Duration) {
        self.elapsed += delta;
        self.progress = (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0);
    }

    fn render(&self, ui: &mut Ui) {
        let painter = ui.painter();
        
        // Draw animated connection line
        let current_pos = Pos2::new(
            self.from_pos.x + (self.to_pos.x - self.from_pos.x) * self.progress,
            self.from_pos.y + (self.to_pos.y - self.from_pos.y) * self.progress,
        );
        
        let stroke_width = 3.0 * (1.0 - self.progress);
        let alpha = ((1.0 - self.progress) * 255.0) as u8;
        
        painter.line_segment([self.from_pos, current_pos], Stroke::new(stroke_width, Color32::from_rgba_premultiplied(0, 255, 0, alpha)));
    }

    fn is_complete(&self) -> bool {
        self.progress >= 1.0
    }

    fn progress(&self) -> f32 {
        self.progress
    }
}

/// Simulation running indicator animation
#[derive(Debug, Clone)]
struct SimulationIndicatorAnimation {
    center: Pos2,
    duration: Duration,
    elapsed: Duration,
    rotation: f32,
}

impl SimulationIndicatorAnimation {
    fn update(&mut self, delta: Duration) {
        self.elapsed += delta;
        let rotation_speed = 360.0; // degrees per second
        self.rotation += delta.as_secs_f32() * rotation_speed;
    }

    fn render(&self, ui: &mut Ui) {
        let painter = ui.painter();
        
        // Draw rotating indicator
        let radius = 15.0;
        let num_spokes = 8;
        
        for i in 0..num_spokes {
            let angle = (i as f32 * 360.0 / num_spokes as f32 + self.rotation) * std::f32::consts::PI / 180.0;
            let x1 = self.center.x + angle.cos() * radius * 0.5;
            let y1 = self.center.y + angle.sin() * radius * 0.5;
            let x2 = self.center.x + angle.cos() * radius;
            let y2 = self.center.y + angle.sin() * radius;
            
            let intensity = ((i as f32 * 360.0 / num_spokes as f32 + self.rotation) % 360.0) / 360.0;
            let alpha = (intensity * 255.0) as u8;
            
            painter.line_segment([Pos2::new(x1, y1), Pos2::new(x2, y2)], Stroke::new(2.0, Color32::from_rgba_premultiplied(0, 255, 0, alpha)));
        }
    }

    fn is_complete(&self) -> bool {
        false // This animation runs continuously
    }

    fn progress(&self) -> f32 {
        (self.elapsed.as_secs_f32() % 1.0) / 1.0
    }
}

/// Particle for current flow visualization
#[derive(Debug, Clone)]
struct Particle {
    position: f32,
    speed: f32,
    size: f32,
    color: Color32,
}

/// Animation configuration
#[derive(Debug, Clone)]
pub struct AnimationConfig {
    pub enable_current_flow: bool,
    pub enable_voltage_levels: bool,
    pub enable_selection_highlight: bool,
    pub enable_connections: bool,
    pub current_flow_speed: f32,
    pub voltage_pulse_frequency: f32,
    pub selection_pulse_frequency: f32,
    pub connection_duration: Duration,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            enable_current_flow: true,
            enable_voltage_levels: true,
            enable_selection_highlight: true,
            enable_connections: true,
            current_flow_speed: 1.0,
            voltage_pulse_frequency: 2.0,
            selection_pulse_frequency: 3.0,
            connection_duration: Duration::from_millis(500),
        }
    }
}

impl AnimationConfig {
    pub fn minimal() -> Self {
        Self {
            enable_current_flow: false,
            enable_voltage_levels: false,
            enable_selection_highlight: false,
            enable_connections: false,
            ..Self::default()
        }
    }

    pub fn full() -> Self {
        Self {
            enable_current_flow: true,
            enable_voltage_levels: true,
            enable_selection_highlight: true,
            enable_connections: true,
            current_flow_speed: 2.0,
            voltage_pulse_frequency: 4.0,
            selection_pulse_frequency: 5.0,
            connection_duration: Duration::from_millis(300),
        }
    }
}