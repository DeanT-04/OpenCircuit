//! Schematic renderer for circuit visualization
//! 
//! Provides high-performance 2D rendering of circuit schematics with
//! real-time updates and interactive features.

use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
use opencircuit_core::models::Circuit;
use opencircuit_circuit::components::Component;
use std::collections::HashMap;

use crate::primitives::*;
use crate::styles::*;

/// Main schematic renderer for circuit visualization
pub struct SchematicRenderer {
    /// Current zoom level (1.0 = 100%)
    zoom: f32,
    /// Pan offset from origin
    pan: Vec2,
    /// Grid size in pixels
    grid_size: f32,
    /// Whether to show grid
    show_grid: bool,
    /// Component positions and orientations
    component_positions: HashMap<String, ComponentPosition>,
    /// Wire paths
    wires: Vec<Wire>,
    /// Selection state
    selection: SelectionState,
    /// Animation state for real-time updates
    animation_state: AnimationState,
}

#[derive(Debug, Clone, Copy)]
pub struct ComponentPosition {
    pub position: Pos2,
    pub rotation: f32,
    pub mirrored: bool,
}

#[derive(Debug, Clone)]
pub struct Wire {
    pub start: Pos2,
    pub end: Pos2,
    pub net_name: Option<String>,
    pub color: Color32,
}

#[derive(Debug, Default)]
pub struct SelectionState {
    pub selected_components: Vec<String>,
    pub selected_wires: Vec<usize>,
    pub hover_component: Option<String>,
}

#[derive(Debug, Default)]
pub struct AnimationState {
    pub simulation_active: bool,
    pub voltages: HashMap<String, f64>,
    pub currents: HashMap<String, f64>,
    pub time: f32,
}

impl SchematicRenderer {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
            grid_size: 20.0,
            show_grid: true,
            component_positions: HashMap::new(),
            wires: Vec::new(),
            selection: SelectionState::default(),
            animation_state: AnimationState::default(),
        }
    }

    /// Render the complete schematic
    pub fn render(&mut self, ui: &mut Ui, circuit: &Circuit) -> Response {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        
        // Transform coordinates
        let to_screen = egui::emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );
        
        // Draw background
        self.draw_background(&painter, &response.rect);
        
        // Draw grid
        if self.show_grid {
            self.draw_grid(&painter, &response.rect);
        }
        
        // Draw components
        self.draw_components(&painter, circuit, &to_screen);
        
        // Draw wires
        self.draw_wires(&painter, &to_screen);
        
        // Draw selection highlights
        self.draw_selection_highlights(&painter, &to_screen);
        
        // Draw simulation results
        if self.animation_state.simulation_active {
            self.draw_simulation_results(&painter, circuit, &to_screen);
        }
        
        response
    }

    fn draw_background(&self, painter: &egui::Painter, rect: &Rect) {
        painter.rect_filled(
            *rect,
            egui::Rounding::none(),
            Color32::from_gray(245),
        );
    }

    fn draw_grid(&self, painter: &egui::Painter, rect: &Rect) {
        let stroke = Stroke::new(0.5, Color32::from_gray(220));
        
        // Vertical lines
        let mut x = rect.left();
        while x <= rect.right() {
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                stroke,
            );
            x += self.grid_size * self.zoom;
        }
        
        // Horizontal lines
        let mut y = rect.top();
        while y <= rect.bottom() {
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                stroke,
            );
            y += self.grid_size * self.zoom;
        }
    }

    fn draw_components(&self, painter: &egui::Painter, circuit: &Circuit, to_screen: &egui::emath::RectTransform) {
        for component in &circuit.components {
            if let Some(position) = self.component_positions.get(&component.id) {
                let screen_pos = to_screen.transform_pos(position.position);
                self.draw_component(painter, component, screen_pos, position.rotation);
            }
        }
    }

    fn draw_component(&self, painter: &egui::Painter, component: &Component, pos: Pos2, rotation: f32) {
        match component.component_type.as_str() {
            "resistor" => self.draw_resistor(painter, pos, rotation),
            "capacitor" => self.draw_capacitor(painter, pos, rotation),
            "inductor" => self.draw_inductor(painter, pos, rotation),
            "voltage_source" => self.draw_voltage_source(painter, pos, rotation),
            "current_source" => self.draw_current_source(painter, pos, rotation),
            "ground" => self.draw_ground(painter, pos),
            _ => self.draw_generic_component(painter, pos, rotation),
        }
    }

    fn draw_resistor(&self, painter: &egui::Painter, pos: Pos2, rotation: f32) {
        let rect = Rect::from_center_size(pos, egui::vec2(60.0, 20.0));
        let stroke = Stroke::new(2.0, Color32::BLACK);
        
        // Draw zigzag resistor symbol
        let points = vec![
            rect.left_center(),
            rect.left_center() + egui::vec2(10.0, 0.0),
            rect.left_center() + egui::vec2(15.0, 8.0),
            rect.left_center() + egui::vec2(25.0, -8.0),
            rect.left_center() + egui::vec2(35.0, 8.0),
            rect.left_center() + egui::vec2(45.0, -8.0),
            rect.left_center() + egui::vec2(50.0, 0.0),
            rect.right_center(),
        ];
        
        for i in 0..points.len()-1 {
            painter.line_segment([points[i], points[i+1]], stroke);
        }
    }

    fn draw_capacitor(&self, painter: &egui::Painter, pos: Pos2, rotation: f32) {
        let rect = Rect::from_center_size(pos, egui::vec2(40.0, 20.0));
        let stroke = Stroke::new(2.0, Color32::BLACK);
        
        // Draw capacitor plates
        painter.line_segment(
            [rect.left_center(), rect.left_center() + egui::vec2(15.0, 0.0)],
            stroke,
        );
        painter.line_segment(
            [rect.right_center() - egui::vec2(15.0, 0.0), rect.right_center()],
            stroke,
        );
        
        // Draw plates
        painter.line_segment(
            [rect.left_center() + egui::vec2(15.0, -10.0), rect.left_center() + egui::vec2(15.0, 10.0)],
            stroke,
        );
        painter.line_segment(
            [rect.right_center() - egui::vec2(15.0, -10.0), rect.right_center() - egui::vec2(15.0, 10.0)],
            stroke,
        );
    }

    fn draw_inductor(&self, painter: &egui::Painter, pos: Pos2, rotation: f32) {
        let rect = Rect::from_center_size(pos, egui::vec2(50.0, 20.0));
        let stroke = Stroke::new(2.0, Color32::BLACK);
        
        // Draw inductor coils
        let radius = 8.0;
        let center = rect.center();
        for i in 0..4 {
            let x = center.x - 20.0 + (i as f32 * 10.0);
            painter.circle_stroke(
                egui::pos2(x, center.y),
                radius,
                stroke,
            );
        }
        
        // Connect ends
        painter.line_segment(
            [rect.left_center(), rect.left_center() + egui::vec2(8.0, 0.0)],
            stroke,
        );
        painter.line_segment(
            [rect.right_center() - egui::vec2(8.0, 0.0), rect.right_center()],
            stroke,
        );
    }

    fn draw_voltage_source(&self, painter: &egui::Painter, pos: Pos2, rotation: f32) {
        let rect = Rect::from_center_size(pos, egui::vec2(40.0, 40.0));
        let stroke = Stroke::new(2.0, Color32::BLACK);
        
        // Draw circle
        painter.circle_stroke(rect.center(), 20.0, stroke);
        
        // Draw + and - symbols
        painter.line_segment(
            [rect.center() - egui::vec2(0.0, 10.0), rect.center() - egui::vec2(0.0, 5.0)],
            stroke,
        );
        painter.line_segment(
            [rect.center() - egui::vec2(5.0, 7.5), rect.center() + egui::vec2(5.0, 7.5)],
            stroke,
        );
        painter.line_segment(
            [rect.center() + egui::vec2(0.0, 5.0), rect.center() + egui::vec2(0.0, 10.0)],
            stroke,
        );
    }

    fn draw_current_source(&self, painter: &egui::Painter, pos: Pos2, rotation: f32) {
        let rect = Rect::from_center_size(pos, egui::vec2(40.0, 40.0));
        let stroke = Stroke::new(2.0, Color32::BLACK);
        
        // Draw circle
        painter.circle_stroke(rect.center(), 20.0, stroke);
        
        // Draw arrow
        let arrow_start = rect.center() - egui::vec2(0.0, 10.0);
        let arrow_end = rect.center() + egui::vec2(0.0, 10.0);
        painter.arrow(arrow_start, arrow_end - arrow_start, stroke);
    }

    fn draw_ground(&self, painter: &egui::Painter, pos: Pos2) {
        let stroke = Stroke::new(2.0, Color32::BLACK);
        
        // Draw ground symbol
        let lines = vec![
            (egui::pos2(pos.x - 10.0, pos.y), egui::pos2(pos.x + 10.0, pos.y)),
            (egui::pos2(pos.x - 7.0, pos.y + 5.0), egui::pos2(pos.x + 7.0, pos.y + 5.0)),
            (egui::pos2(pos.x - 4.0, pos.y + 10.0), egui::pos2(pos.x + 4.0, pos.y + 10.0)),
        ];
        
        for (start, end) in lines {
            painter.line_segment([start, end], stroke);
        }
    }

    fn draw_generic_component(&self, painter: &egui::Painter, pos: Pos2, rotation: f32) {
        let rect = Rect::from_center_size(pos, egui::vec2(40.0, 20.0));
        let stroke = Stroke::new(2.0, Color32::BLACK);
        
        // Draw rectangle
        painter.rect_stroke(rect, egui::Rounding::none(), stroke);
        
        // Draw question mark
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "?",
            egui::FontId::proportional(16.0),
            Color32::BLACK,
        );
    }

    fn draw_wires(&self, painter: &egui::Painter, to_screen: &egui::emath::RectTransform) {
        for wire in &self.wires {
            let start = to_screen.transform_pos(wire.start);
            let end = to_screen.transform_pos(wire.end);
            
            let stroke = Stroke::new(2.0, wire.color);
            painter.line_segment([start, end], stroke);
        }
    }

    fn draw_selection_highlights(&self, painter: &egui::Painter, to_screen: &egui::emath::RectTransform) {
        // Draw selection boxes
        for component_id in &self.selection.selected_components {
            if let Some(position) = self.component_positions.get(component_id) {
                let screen_pos = to_screen.transform_pos(position.position);
                let rect = Rect::from_center_size(screen_pos, egui::vec2(70.0, 40.0));
                
                painter.rect_stroke(
                    rect,
                    egui::Rounding::same(4.0),
                    Stroke::new(2.0, Color32::from_rgb(0, 120, 255)),
                );
            }
        }
    }

    fn draw_simulation_results(&self, painter: &egui::Painter, circuit: &Circuit, to_screen: &egui::emath::RectTransform) {
        // Draw voltage indicators
        for (node_id, voltage) in &self.animation_state.voltages {
            if let Some(position) = self.component_positions.get(node_id) {
                let screen_pos = to_screen.transform_pos(position.position);
                
                let color = if *voltage > 0.0 {
                    Color32::from_rgb(0, 200, 0)
                } else {
                    Color32::from_rgb(200, 0, 0)
                };
                
                painter.text(
                    screen_pos + egui::vec2(0.0, -25.0),
                    egui::Align2::CENTER_CENTER,
                    format!("{:.2}V", voltage),
                    egui::FontId::monospace(12.0),
                    color,
                );
            }
        }
    }

    /// Update simulation results
    pub fn update_simulation_results(&mut self, voltages: HashMap<String, f64>, currents: HashMap<String, f64>) {
        self.animation_state.voltages = voltages;
        self.animation_state.currents = currents;
        self.animation_state.time += 0.1;
    }

    /// Toggle simulation animation
    pub fn toggle_simulation(&mut self) {
        self.animation_state.simulation_active = !self.animation_state.simulation_active;
    }

    /// Set component position
    pub fn set_component_position(&mut self, component_id: String, position: ComponentPosition) {
        self.component_positions.insert(component_id, position);
    }

    /// Add wire
    pub fn add_wire(&mut self, wire: Wire) {
        self.wires.push(wire);
    }

    /// Clear all components and wires
    pub fn clear(&mut self) {
        self.component_positions.clear();
        self.wires.clear();
        self.selection = SelectionState::default();
    }
}