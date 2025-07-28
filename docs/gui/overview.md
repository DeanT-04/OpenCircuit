---
title: GUI Framework Overview
description: Comprehensive guide to Tauri and egui for OpenCircuit's user interface
last_updated: 2025-01-27
tags: [gui, tauri, egui, desktop, ui]
context_id: gui.overview.main
---

# 🖥️ GUI Framework Overview

OpenCircuit uses a modern GUI stack combining **Tauri** for the application framework and **egui** for the immediate-mode user interface.

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────┐
│           Tauri Application         │
├─────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐   │
│  │    egui     │  │   Web View  │   │
│  │   (Rust)    │  │ (Optional)  │   │
│  └─────────────┘  └─────────────┘   │
├─────────────────────────────────────┤
│         Rust Backend Core          │
│    (Circuit Sim, PCB, AI, etc.)    │
└─────────────────────────────────────┘
```

## 🚀 Tauri Framework

### Key Features
- **Cross-platform** - Windows, macOS, Linux
- **Small bundle size** - No Electron overhead
- **Native performance** - Rust backend
- **Security-first** - Sandboxed frontend
- **Plugin ecosystem** - Extensible architecture

### Core Components
- **Commands** - Rust functions callable from frontend
- **Events** - Bidirectional communication
- **File System** - Secure file operations
- **Window Management** - Multi-window support
- **System Integration** - Native OS features

```rust
// @context_id: gui.tauri.commands
// @purpose: Define Tauri commands for circuit operations
use tauri::command;

#[command]
async fn load_circuit(path: String) -> Result<Circuit, String> {
    Circuit::load_from_file(&path)
        .map_err(|e| e.to_string())
}

#[command]
async fn simulate_circuit(circuit: Circuit) -> Result<SimulationResult, String> {
    circuit.simulate()
        .map_err(|e| e.to_string())
}
```

## 🎨 egui Immediate Mode GUI

### Philosophy
- **Immediate mode** - UI state in application logic
- **Retained mode rendering** - Efficient GPU usage
- **Simple API** - Easy to learn and use
- **Portable** - Runs on multiple backends

### Core Concepts
- **Context** - Global state and settings
- **UI** - Layout and widget container
- **Response** - User interaction feedback
- **Painter** - Low-level drawing operations

```rust
// @context_id: gui.egui.circuit_editor
// @purpose: Basic circuit editor UI layout
use egui::{Context, Ui, Response};

pub struct CircuitEditor {
    circuit: Circuit,
    selected_component: Option<ComponentId>,
    zoom: f32,
    pan_offset: egui::Vec2,
}

impl CircuitEditor {
    pub fn show(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_toolbar(ui);
            self.show_canvas(ui);
            self.show_properties_panel(ui);
        });
    }
    
    fn show_canvas(&mut self, ui: &mut Ui) {
        let response = ui.allocate_response(
            ui.available_size(),
            egui::Sense::click_and_drag()
        );
        
        if response.dragged() {
            self.pan_offset += response.drag_delta();
        }
        
        // Draw circuit components
        self.draw_circuit(ui, &response);
    }
}
```

## 🔧 Integration Benefits

### Performance
- **Native rendering** - Direct GPU access
- **Efficient updates** - Only redraw when needed
- **Memory safety** - Rust's ownership system
- **Parallel processing** - Multi-threaded backend

### Development Experience
- **Hot reload** - Fast iteration cycles
- **Type safety** - Compile-time error checking
- **Debugging** - Native debugging tools
- **Testing** - Unit and integration tests

## 📦 Key Dependencies

### Tauri Ecosystem
```toml
[dependencies]
tauri = { version = "1.0", features = ["api-all"] }
tauri-plugin-fs-extra = "1.0"
tauri-plugin-window-state = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### egui Ecosystem
```toml
[dependencies]
egui = "0.24"
eframe = "0.24"
egui_extras = "0.24"
egui_plot = "0.24"
```

## 🎯 OpenCircuit-Specific Features

### Circuit Canvas
- **Infinite canvas** - Pan and zoom
- **Component library** - Drag-and-drop components
- **Wire routing** - Automatic and manual routing
- **Selection tools** - Multi-select and grouping

### Property Panels
- **Component properties** - Real-time editing
- **Simulation parameters** - Analysis settings
- **Design rules** - PCB constraints
- **Layer management** - Multi-layer visualization

### Tool Integration
- **Simulation controls** - Start/stop/pause
- **AI assistant** - Context-aware help
- **File management** - Project organization
- **Export options** - Multiple formats

## 🔗 Quick Links

- [Tauri Documentation](tauri.md)
- [egui Framework Guide](egui.md)
- [Integration Examples](integration.md)
- [Performance Optimization](performance.md)

---

*Context ID: gui.overview.main*