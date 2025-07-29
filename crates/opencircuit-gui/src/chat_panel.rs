//! Chat panel implementation for OpenCircuit
//! 
//! This module provides the chat interface where users can interact with the AI assistant
//! for circuit design guidance, component recommendations, and technical support.

use crate::gui::{ChatMessage, AppState};
use egui::{Context, Ui, ScrollArea, TextEdit, Button, RichText, Color32, Frame, Margin};
use chrono::{DateTime, Utc};

/// Chat panel widget for the OpenCircuit application
pub struct ChatPanel {
    /// Current message being typed by the user
    current_input: String,
    /// Auto-scroll to bottom when new messages arrive
    auto_scroll: bool,
    /// Maximum number of messages to display
    max_messages: usize,
}

impl Default for ChatPanel {
    fn default() -> Self {
        Self {
            current_input: String::new(),
            auto_scroll: true,
            max_messages: 1000,
        }
    }
}

impl ChatPanel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the chat panel UI
    pub fn show(&mut self, ctx: &Context, ui: &mut Ui, state: &mut AppState) {
        ui.vertical(|ui| {
            // Chat header
            self.show_header(ui);
            
            // Message history area
            self.show_message_history(ui, &state.chat_messages);
            
            // Input area
            self.show_input_area(ui, state);
        });
    }

    fn show_header(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("💬 AI Assistant");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("🗑️ Clear").clicked() {
                    // Clear button functionality will be handled by parent
                }
                ui.label(format!("Auto-scroll: {}", if self.auto_scroll { "✅" } else { "❌" }));
            });
        });
        ui.separator();
    }

    fn show_message_history(&mut self, ui: &mut Ui, messages: &[ChatMessage]) {
        let available_height = ui.available_height() - 80.0; // Reserve space for input
        
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .max_height(available_height)
            .stick_to_bottom(self.auto_scroll)
            .show(ui, |ui| {
                if messages.is_empty() {
                    self.show_welcome_message(ui);
                } else {
                    for message in messages.iter().rev().take(self.max_messages).rev() {
                        self.show_message(ui, message);
                        ui.add_space(8.0);
                    }
                }
            });
    }

    fn show_welcome_message(&self, ui: &mut Ui) {
        Frame::none()
            .fill(Color32::from_gray(240))
            .rounding(8.0)
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(RichText::new("🤖 Welcome to OpenCircuit AI Assistant!").size(18.0).strong());
                    ui.add_space(10.0);
                    ui.label("I'm here to help you with:");
                    ui.label("• Circuit design and analysis");
                    ui.label("• Component selection and recommendations");
                    ui.label("• PCB layout optimization");
                    ui.label("• Technical questions and troubleshooting");
                    ui.add_space(10.0);
                    ui.label(RichText::new("Start by asking me anything about electronics!").italics());
                    ui.add_space(20.0);
                });
            });
    }

    fn show_message(&self, ui: &mut Ui, message: &ChatMessage) {
        let (bg_color, text_color, alignment) = if message.is_user {
            (Color32::from_rgb(0, 120, 215), Color32::WHITE, egui::Layout::right_to_left(egui::Align::Top))
        } else {
            (Color32::from_gray(230), Color32::BLACK, egui::Layout::left_to_right(egui::Align::Top))
        };

        ui.with_layout(alignment, |ui| {
            let max_width = ui.available_width() * 0.75;
            
            Frame::none()
                .fill(bg_color)
                .rounding(12.0)
                .inner_margin(Margin::symmetric(12.0, 8.0))
                .show(ui, |ui| {
                    ui.set_max_width(max_width);
                    
                    // Message content
                    ui.label(RichText::new(&message.content).color(text_color));
                    
                    // Timestamp
                    let time_str = message.timestamp.format("%H:%M").to_string();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Bottom), |ui| {
                        ui.label(RichText::new(time_str).size(10.0).color(text_color.gamma_multiply(0.7)));
                    });
                });
        });
    }

    fn show_input_area(&mut self, ui: &mut Ui, state: &mut AppState) {
        ui.separator();
        ui.add_space(4.0);
        
        ui.horizontal(|ui| {
            // Text input field
            let text_edit = TextEdit::multiline(&mut self.current_input)
                .desired_width(ui.available_width() - 80.0)
                .desired_rows(2)
                .hint_text("Type your message here...");
            
            let response = ui.add(text_edit);
            
            // Send button
            ui.vertical(|ui| {
                let send_button = Button::new("📤 Send")
                    .min_size(egui::Vec2::new(70.0, 40.0));
                
                let can_send = !self.current_input.trim().is_empty();
                ui.add_enabled_ui(can_send, |ui| {
                    if ui.add(send_button).clicked() || 
                       (response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl)) {
                        self.send_message(state);
                    }
                });
                
                if ui.small_button("🔄").clicked() {
                    self.auto_scroll = !self.auto_scroll;
                }
            });
        });
        
        ui.label(RichText::new("💡 Tip: Press Ctrl+Enter to send").size(10.0).weak());
    }

    fn send_message(&mut self, state: &mut AppState) {
        if self.current_input.trim().is_empty() {
            return;
        }

        // Add user message
        let user_message = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            content: self.current_input.trim().to_string(),
            is_user: true,
            timestamp: Utc::now(),
        };
        state.chat_messages.push(user_message);

        // Generate AI response (placeholder for now)
        let ai_response = self.generate_ai_response(&self.current_input);
        let ai_message = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            content: ai_response,
            is_user: false,
            timestamp: Utc::now(),
        };
        state.chat_messages.push(ai_message);

        // Clear input and enable auto-scroll
        self.current_input.clear();
        self.auto_scroll = true;
    }

    fn generate_ai_response(&self, user_input: &str) -> String {
        // Placeholder AI response logic
        // In the next task, this will be replaced with actual OpenAI API integration
        
        let input_lower = user_input.to_lowercase();
        
        if input_lower.contains("resistor") || input_lower.contains("resistance") {
            "🔧 For resistor selection, I recommend considering the power rating, tolerance, and temperature coefficient. What's your target resistance value and power requirement?".to_string()
        } else if input_lower.contains("capacitor") {
            "⚡ Capacitors come in many types - ceramic, electrolytic, tantalum, etc. The choice depends on your application. What voltage and capacitance range are you looking for?".to_string()
        } else if input_lower.contains("circuit") || input_lower.contains("design") {
            "🔌 I'd be happy to help with your circuit design! Could you tell me more about what you're trying to build? What's the intended function and any specific requirements?".to_string()
        } else if input_lower.contains("pcb") || input_lower.contains("layout") {
            "📋 PCB layout is crucial for circuit performance. Key considerations include trace width, via placement, ground planes, and component spacing. What type of circuit are you laying out?".to_string()
        } else if input_lower.contains("hello") || input_lower.contains("hi") {
            "👋 Hello! I'm your OpenCircuit AI assistant. I'm here to help with circuit design, component selection, PCB layout, and any electronics questions you might have. What can I help you with today?".to_string()
        } else {
            format!("🤖 I understand you're asking about: \"{}\"\n\nThis is a placeholder response. In the full implementation, I'll provide detailed technical guidance based on my knowledge of electronics, circuit design, and component databases. How can I help you further with your project?", user_input)
        }
    }

    /// Clear all chat messages
    pub fn clear_messages(&mut self, state: &mut AppState) {
        state.chat_messages.clear();
    }

    /// Set auto-scroll behavior
    pub fn set_auto_scroll(&mut self, enabled: bool) {
        self.auto_scroll = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_panel_creation() {
        let panel = ChatPanel::new();
        assert!(panel.current_input.is_empty());
        assert!(panel.auto_scroll);
        assert_eq!(panel.max_messages, 1000);
    }

    #[test]
    fn test_ai_response_generation() {
        let panel = ChatPanel::new();
        
        let response = panel.generate_ai_response("I need a resistor");
        assert!(response.contains("resistor"));
        
        let response = panel.generate_ai_response("Hello");
        assert!(response.contains("Hello"));
    }
}