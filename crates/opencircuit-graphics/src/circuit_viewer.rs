//! Circuit viewer with real-time updates and interactive features
//! 
//! Provides the main interface for circuit visualization with support for
//! real-time simulation updates, interactive editing, and responsive design.

use egui::{CentralPanel, Context, Response, SidePanel, Ui, Vec2};
use opencircuit_core::models::Circuit;
use opencircuit_simulation::CircuitSimulator;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::schematic_renderer::{SchematicRenderer, Wire};
use crate::styles::CircuitStyle;

/// Main circuit viewer widget
pub struct CircuitViewer {
    renderer: SchematicRenderer,
    circuit: Option<Circuit>,
    simulator: Arc<Mutex<Option<CircuitSimulator>>>,
    style: CircuitStyle,
    show_toolbar: bool,
    show_properties: bool,
    auto_simulate: bool,
    simulation_running: bool,
}

impl CircuitViewer {
    pub fn new() -> Self {
        Self {
            renderer: SchematicRenderer::new(),
            circuit: None,
            simulator: Arc::new(Mutex::new(None)),
            style: CircuitStyle::default(),
            show_toolbar: true,
            show_properties: true,
            auto_simulate: false,
            simulation_running: false,
        }
    }

    /// Show the complete circuit viewer with all panels
    pub fn show(&mut self, ctx: &Context) {
        // Top toolbar
        if self.show_toolbar {
            self.show_toolbar(ctx);
        }

        // Main circuit canvas
        CentralPanel::default().show(ctx, |ui| {
            self.show_circuit_canvas(ui);
        });

        // Properties panel
        if self.show_properties {
            self.show_properties_panel(ctx);
        }
    }

    /// Show only the circuit canvas without panels
    pub fn show_circuit_canvas(&mut self, ui: &mut Ui) -> Response {
        let available_rect = ui.available_rect_before_wrap();
        
        // Create toolbar for the canvas
        let response = egui::Frame::none()
            .fill(egui::Color32::from_gray(240))
            .rounding(8.0)
            .inner_margin(4.0)
            .show(ui, |ui| {
                self.show_canvas_toolbar(ui);
                ui.separator();
                self.show_canvas_content(ui)
            });
        
        response.response
    }

    fn show_toolbar(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("circuit_toolbar")
            .exact_height(40.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("🔌 Circuit Designer");
                    ui.separator();
                    
                    if ui.button("📁 Open").clicked() {
                        self.open_circuit();
                    }
                    
                    if ui.button("💾 Save").clicked() {
                        self.save_circuit();
                    }
                    
                    ui.separator();
                    
                    if ui.button("➕ Add Component").clicked() {
                        self.show_add_component_menu();
                    }
                    
                    if ui.button("✏️ Wire Tool").clicked() {
                        self.toggle_wire_tool();
                    }
                    
                    ui.separator();
                    
                    if ui.button("▶️ Run Simulation").clicked() {
                        self.run_simulation();
                    }
                    
                    if ui.button("⏹️ Stop").clicked() {
                        self.stop_simulation();
                    }
                    
                    ui.separator();
                    
                    ui.checkbox(&mut self.auto_simulate, "Auto Simulate");
                    
                    ui.separator();
                    
                    // Zoom controls
                    if ui.button("🔍+").clicked() {
                        self.zoom_in();
                    }
                    
                    if ui.button("🔍-").clicked() {
                        self.zoom_out();
                    }
                    
                    if ui.button("🔄 Reset Zoom").clicked() {
                        self.reset_zoom();
                    }
                    
                    ui.separator();
                    
                    ui.checkbox(&mut self.show_properties, "Show Properties");
                });
            });
    }

    fn show_canvas_toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Canvas:");
            
            if ui.button("🔍+").clicked() {
                self.zoom_in();
            }
            
            if ui.button("🔍-").clicked() {
                self.zoom_out();
            }
            
            if ui.button("🔄 Reset").clicked() {
                self.reset_zoom();
            }
            
            ui.separator();
            
            if ui.button("📏 Grid").clicked() {
                self.toggle_grid();
            }
            
            ui.separator();
            
            if ui.button("🧹 Clear").clicked() {
                self.clear_circuit();
            }
        });
    }

    fn show_canvas_content(&mut self, ui: &mut Ui) -> Response {
        let response = if let Some(circuit) = &self.circuit {
            // Render the actual circuit
            self.renderer.render(ui, circuit)
        } else {
            // Show placeholder
            self.show_empty_canvas(ui)
        };

        // Handle canvas interactions
        if response.dragged() {
            self.handle_canvas_drag(response.drag_delta());
        }

        if response.hovered() {
            self.handle_canvas_hover(response.hover_pos());
        }

        response
    }

    fn show_empty_canvas(&mut self, ui: &mut Ui) -> Response {
        let response = ui.allocate_response(ui.available_size(), egui::Sense::click());
        
        let rect = response.rect;
        let painter = ui.painter();
        
        // Draw background
        painter.rect_filled(
            rect,
            egui::Rounding::same(8.0),
            egui::Color32::from_gray(250),
        );
        
        // Draw grid
        self.renderer.draw_grid(&painter, &rect);
        
        // Draw placeholder text
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "🔌 Circuit Canvas\n\nStart by:\n• Adding components from the toolbar\n• Drawing wires between components\n• Running simulations",
            egui::FontId::proportional(16.0),
            egui::Color32::from_gray(120),
        );
        
        response
    }

    fn show_properties_panel(&mut self, ctx: &Context) {
        SidePanel::right("properties_panel")
            .resizable(true)
            .default_width(250.0)
            .width_range(200.0..=400.0)
            .show(ctx, |ui| {
                ui.heading("⚙️ Properties");
                ui.separator();
                
                if let Some(circuit) = &self.circuit {
                    self.show_circuit_properties(ui, circuit);
                } else {
                    ui.label("No circuit loaded");
                }
                
                ui.separator();
                
                // Simulation results
                if self.simulation_running {
                    self.show_simulation_results(ui);
                }
            });
    }

    fn show_circuit_properties(&mut self, ui: &mut Ui, circuit: &Circuit) {
        egui::Grid::new("circuit_properties")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                ui.label("Name:");
                ui.label(&circuit.name);
                ui.end_row();
                
                ui.label("Components:");
                ui.label(circuit.components.len().to_string());
                ui.end_row();
                
                ui.label("Nets:");
                ui.label(circuit.nets.len().to_string());
                ui.end_row();
            });
    }

    fn show_simulation_results(&mut self, ui: &mut Ui) {
        ui.heading("📊 Simulation Results");
        ui.separator();
        
        // Placeholder for simulation results
        ui.label("🔄 Simulation running...");
        ui.add_space(10.0);
        
        ui.label("Voltage sources:");
        ui.label("• V1: 5.0V");
        ui.label("• V2: 3.3V");
        
        ui.add_space(10.0);
        ui.label("Currents:");
        ui.label("• I(R1): 1.2mA");
        ui.label("• I(R2): 0.8mA");
    }

    // Action handlers
    fn open_circuit(&mut self) {
        // TODO: Implement file dialog
        println!("Open circuit file");
    }

    fn save_circuit(&mut self) {
        // TODO: Implement save dialog
        println!("Save circuit file");
    }

    fn show_add_component_menu(&mut self) {
        println!("Show add component menu");
    }

    fn toggle_wire_tool(&mut self) {
        println!("Toggle wire tool");
    }

    fn run_simulation(&mut self) {
        self.simulation_running = true;
        self.renderer.toggle_simulation();
        println!("Starting simulation...");
    }

    fn stop_simulation(&mut self) {
        self.simulation_running = false;
        self.renderer.toggle_simulation();
        println!("Stopping simulation...");
    }

    fn zoom_in(&mut self) {
        // TODO: Implement zoom
        println!("Zoom in");
    }

    fn zoom_out(&mut self) {
        // TODO: Implement zoom
        println!("Zoom out");
    }

    fn reset_zoom(&mut self) {
        // TODO: Implement zoom reset
        println!("Reset zoom");
    }

    fn toggle_grid(&mut self) {
        // TODO: Implement grid toggle
        println!("Toggle grid");
    }

    fn clear_circuit(&mut self) {
        self.circuit = None;
        self.renderer.clear();
        println!("Clear circuit");
    }

    fn handle_canvas_drag(&mut self, delta: Vec2) {
        // TODO: Implement canvas panning
        println!("Canvas drag: {:?}", delta);
    }

    fn handle_canvas_hover(&mut self, pos: Option<egui::Pos2>) {
        if let Some(_pos) = pos {
            // TODO: Implement hover effects
        }
    }

    /// Load a circuit into the viewer
    pub fn load_circuit(&mut self, circuit: Circuit) {
        self.circuit = Some(circuit);
        
        // Initialize component positions
        // TODO: Implement proper layout algorithm
        if let Some(circuit) = &self.circuit {
            for (i, component) in circuit.components.iter().enumerate() {
                let x = 100.0 + (i as f32 % 5) as f32 * 100.0;
                let y = 100.0 + (i as f32 / 5.0) as f32 * 100.0;
                
                use crate::schematic_renderer::ComponentPosition;
                self.renderer.set_component_position(
                    component.id.clone(),
                    ComponentPosition {
                        position: egui::pos2(x, y),
                        rotation: 0.0,
                        mirrored: false,
                    },
                );
            }
        }
    }

    /// Update simulation results
    pub fn update_simulation(&mut self, voltages: std::collections::HashMap<String, f64>, currents: std::collections::HashMap<String, f64>) {
        self.renderer.update_simulation_results(voltages, currents);
    }

    /// Get current circuit
    pub fn get_circuit(&self) -> Option<&Circuit> {
        self.circuit.as_ref()
    }
}