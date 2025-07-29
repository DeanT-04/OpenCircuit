//! egui-based GUI application for OpenCircuit
//! 
//! This module implements the main egui application with three-panel layout:
//! - Left panel: Chat interface with AI assistant
//! - Center panel: Circuit visualization and editing
//! - Right panel: Research console and component browser

use crate::gui::{AppState, ChatPanel, ResearchStatus};
use crate::ai::ChatHandler;
use crate::OpenCircuitResult;
use eframe::egui::{self, Context, CentralPanel, SidePanel, TopBottomPanel, Ui};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Main OpenCircuit egui application
pub struct OpenCircuitEguiApp {
    /// Application state
    state: AppState,
    /// Chat panel widget
    chat_panel: ChatPanel,
    /// AI chat handler
    chat_handler: Arc<Mutex<ChatHandler>>,
    /// Runtime for async operations
    runtime: tokio::runtime::Runtime,
}

impl OpenCircuitEguiApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            state: AppState::default(),
            chat_panel: ChatPanel::new(),
            chat_handler: Arc::new(Mutex::new(ChatHandler::new())),
            runtime: tokio::runtime::Runtime::new().expect("Failed to create tokio runtime"),
        }
    }

    /// Show the left chat panel
    fn show_chat_panel(&mut self, ctx: &Context) {
        SidePanel::left("chat_panel")
            .resizable(true)
            .default_width(350.0)
            .width_range(250.0..=500.0)
            .show(ctx, |ui| {
                ui.heading("💬 AI Assistant");
                ui.separator();
                self.chat_panel.show(ctx, ui, &mut self.state);
            });
    }

    /// Show the center circuit panel
    fn show_circuit_panel(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            self.show_circuit_header(ui);
            ui.separator();
            
            if self.state.current_circuit.is_some() {
                self.show_circuit_canvas(ui);
            } else {
                self.show_circuit_placeholder(ui);
            }
        });
    }

    fn show_circuit_header(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("🔌 Circuit Designer");
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("📁 Open").clicked() {
                    // TODO: Implement file dialog
                }
                if ui.button("💾 Save").clicked() {
                    // TODO: Implement save functionality
                }
                if ui.button("▶️ Simulate").clicked() {
                    // TODO: Implement simulation
                }
            });
        });
    }

    fn show_circuit_canvas(&self, ui: &mut Ui) {
        let available_rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(available_rect, egui::Sense::click_and_drag());
        
        // Draw circuit canvas background
        ui.painter().rect_filled(
            response.rect,
            egui::Rounding::same(4.0),
            egui::Color32::from_gray(250),
        );
        
        // Draw grid
        self.draw_grid(ui, &response.rect);
        
        // Placeholder circuit elements
        ui.painter().text(
            response.rect.center(),
            egui::Align2::CENTER_CENTER,
            "🔌 Circuit Canvas\n\n(Circuit visualization will be implemented in Phase 3)",
            egui::FontId::proportional(16.0),
            egui::Color32::from_gray(120),
        );
    }

    fn show_circuit_placeholder(&self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading("🔌 Welcome to OpenCircuit");
            ui.add_space(20.0);
            
            ui.label("Start by:");
            ui.label("• Asking the AI assistant about your circuit design");
            ui.label("• Opening an existing circuit file");
            ui.label("• Creating a new circuit from scratch");
            
            ui.add_space(30.0);
            
            if ui.button("🆕 New Circuit").clicked() {
                // TODO: Create new circuit
            }
            
            ui.add_space(20.0);
            
            // Sample circuit preview
            egui::Frame::none()
                .fill(egui::Color32::from_gray(240))
                .rounding(8.0)
                .inner_margin(egui::style::Margin::same(20.0))
                .show(ui, |ui| {
                    ui.label("📐 Sample Circuit Preview:");
                    ui.monospace("    VCC");
                    ui.monospace("     |");
                    ui.monospace("   ┌─R1─┐");
                    ui.monospace("   │    │");
                    ui.monospace("  LED   │");
                    ui.monospace("   │    │");
                    ui.monospace("   └────┘");
                    ui.monospace("     |");
                    ui.monospace("    GND");
                });
        });
    }

    fn draw_grid(&self, ui: &Ui, rect: &egui::Rect) {
        let grid_size = 20.0;
        let painter = ui.painter();
        
        // Vertical lines
        let mut x = rect.left();
        while x <= rect.right() {
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                egui::Stroke::new(0.5, egui::Color32::from_gray(220)),
            );
            x += grid_size;
        }
        
        // Horizontal lines
        let mut y = rect.top();
        while y <= rect.bottom() {
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(0.5, egui::Color32::from_gray(220)),
            );
            y += grid_size;
        }
    }

    /// Show the right research panel
    fn show_research_panel(&mut self, ctx: &Context) {
        SidePanel::right("research_panel")
            .resizable(true)
            .default_width(300.0)
            .width_range(200.0..=400.0)
            .show(ctx, |ui| {
                ui.heading("🔍 Research Console");
                ui.separator();
                self.show_research_content(ui);
            });
    }

    fn show_research_content(&mut self, ui: &mut Ui) {
        match self.state.research_status {
            ResearchStatus::Idle => {
                ui.label("🟢 Research system ready");
                ui.add_space(10.0);
                
                if ui.button("🔍 Start Component Research").clicked() {
                    self.state.research_status = ResearchStatus::Searching;
                }
                
                ui.add_space(20.0);
                ui.label("📚 Recent Searches:");
                ui.label("• 555 Timer IC");
                ui.label("• Op-amp LM358");
                ui.label("• Power MOSFET");
            }
            ResearchStatus::Searching => {
                ui.label("🔄 Searching component databases...");
                ui.add_space(10.0);
                
                // Animated progress indicator
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Analyzing specifications...");
                });
                
                // Auto-advance to next state (simulation)
                ui.ctx().request_repaint_after(std::time::Duration::from_millis(2000));
                self.state.research_status = ResearchStatus::Analyzing;
            }
            ResearchStatus::Analyzing => {
                ui.label("🧮 Analyzing results...");
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Processing recommendations...");
                });
                
                // Auto-advance to complete state
                ui.ctx().request_repaint_after(std::time::Duration::from_millis(1500));
                self.state.research_status = ResearchStatus::Complete;
            }
            ResearchStatus::Complete => {
                ui.label("✅ Research complete!");
                ui.add_space(10.0);
                
                ui.label("📋 Found Components:");
                ui.label("• 15 resistors");
                ui.label("• 8 capacitors");
                ui.label("• 3 ICs");
                
                ui.add_space(10.0);
                ui.label("💰 Estimated cost: $12.50");
                ui.label("⚡ Power: 150mW");
                
                ui.add_space(15.0);
                if ui.button("🔄 Reset").clicked() {
                    self.state.research_status = ResearchStatus::Idle;
                }
            }
        }
    }

    /// Show the top menu bar
    fn show_menu_bar(&mut self, ctx: &Context) {
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Circuit").clicked() {
                        // TODO: New circuit
                        ui.close_menu();
                    }
                    if ui.button("Open...").clicked() {
                        // TODO: Open file dialog
                        ui.close_menu();
                    }
                    if ui.button("Save").clicked() {
                        // TODO: Save current circuit
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                
                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Redo").clicked() {
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Simulate", |ui| {
                    if ui.button("Run Simulation").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Stop Simulation").clicked() {
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        ui.close_menu();
                    }
                });
            });
        });
    }
}

impl eframe::App for OpenCircuitEguiApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Show menu bar
        self.show_menu_bar(ctx);
        
        // Show main panels
        self.show_chat_panel(ctx);
        self.show_research_panel(ctx);
        self.show_circuit_panel(ctx);
    }
}

/// Run the egui application
pub fn run_egui_app() -> OpenCircuitResult<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("OpenCircuit - AI-Powered Circuit Design"),
        ..Default::default()
    };

    eframe::run_native(
        "OpenCircuit",
        options,
        Box::new(|cc| Ok(Box::new(OpenCircuitEguiApp::new(cc)))),
    ).map_err(|e| anyhow::anyhow!("Failed to run egui app: {}", e))?;

    Ok(())
}

/// Run the application (chooses between console and egui based on availability)
pub fn run_app() -> OpenCircuitResult<()> {
    // Try to run egui app first, fall back to console if it fails
    match run_egui_app() {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("Failed to start egui app: {}", e);
            eprintln!("Falling back to console interface...");
            crate::gui::app::OpenCircuitApp::run()
        }
    }
}