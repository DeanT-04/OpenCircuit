//! GUI module for the OpenCircuit user interface
//! 
//! This module contains:
//! - egui-based three-panel layout
//! - Chat interface with AI assistant
//! - Circuit visualization
//! - Research console animation

pub mod app;
// Temporarily commented out due to egui dependency issues
// pub mod chat_panel;
// pub mod egui_app;  // Temporarily disabled due to dependency issues

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use opencircuit_core::OpenCircuitError;
use uuid::Uuid;

/// Type alias for GUI-specific results
pub type OpenCircuitResult<T> = Result<T, OpenCircuitError>;

/// Application state that persists across the GUI
#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub chat_messages: Vec<opencircuit_ai::chat_handler::ChatMessage>,
    pub current_circuit: Option<String>, // Placeholder for circuit data
    pub research_status: ResearchStatus,
}

/// Status of the research console
#[derive(Debug, Clone, PartialEq)]
pub enum ResearchStatus {
    Idle,
    Searching,
    Analyzing,
    Complete,
}

impl Default for ResearchStatus {
    fn default() -> Self {
        Self::Idle
    }
}

/// Main OpenCircuit application
pub struct OpenCircuitApp {
    state: AppState,
}

impl OpenCircuitApp {
    pub fn new() -> Self {
        Self {
            state: AppState::default(),
        }
    }

    pub fn run() -> Result<()> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| anyhow::anyhow!("Failed to create tokio runtime: {}", e))?;
        
        rt.block_on(async {
            app::run_app().await
        }).map_err(|e| anyhow::anyhow!("GUI error: {}", e))
    }

    pub fn add_chat_message(&mut self, sender: String, content: String) {
        let message = opencircuit_ai::chat_handler::ChatMessage {
            id: Uuid::new_v4().to_string(),
            content,
            is_user: sender == "User",
            timestamp: Utc::now(),
        };
        self.state.chat_messages.push(message);
    }
}

impl Default for OpenCircuitApp {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export for easy access
// Temporarily commented out due to egui dependency issues
// pub use chat_panel::ChatPanel;
// pub use egui_app::run_egui_app;  // Temporarily disabled

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = OpenCircuitApp::new();
        assert_eq!(app.state.chat_messages.len(), 0);
        assert!(app.state.current_circuit.is_none());
        assert_eq!(app.state.research_status, ResearchStatus::Idle);
    }

    #[test]
    fn test_add_chat_message() {
        let mut app = OpenCircuitApp::new();
        app.add_chat_message("User".to_string(), "Hello".to_string());
        
        assert_eq!(app.state.chat_messages.len(), 1);
        assert_eq!(app.state.chat_messages[0].content, "Hello");
        assert!(app.state.chat_messages[0].is_user);
    }
}