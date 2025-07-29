//! Console-based application interface for OpenCircuit
//! 
//! This module provides a text-based interface for interacting with OpenCircuit
//! features including:
//! - AI chat assistant for circuit design help
//! - Circuit visualization (placeholder)
//! - Research console with status tracking

use std::io::{self, Write};
use crate::ai::ChatHandler;
use crate::gui::{AppState, ChatMessage};
use chrono::Utc;

pub struct ConsoleApp {
    state: AppState,
    chat_handler: ChatHandler,
}

impl ConsoleApp {
    pub fn new() -> Self {
        Self {
            state: AppState::default(),
            chat_handler: ChatHandler::new(),
        }
    }

    pub async fn run(&mut self) -> crate::OpenCircuitResult<()> {
        println!("🔌 Welcome to OpenCircuit - AI-Powered Circuit Design Tool");
        println!("Type 'help' for commands, 'quit' to exit\n");

        loop {
            self.display_menu();
            
            print!("\nSelect option: ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let choice = input.trim();

            match choice {
                "1" => self.chat_interface().await?,
                "2" => self.circuit_view(),
                "3" => self.research_console(),
                "help" => self.show_help(),
                "quit" | "exit" | "q" => {
                    println!("Goodbye! 👋");
                    break;
                }
                _ => println!("Invalid option. Type 'help' for available commands."),
            }
        }

        Ok(())
    }

    fn display_menu(&self) {
        println!("\n═══════════════════════════════════════");
        println!("🔌 OpenCircuit Main Menu");
        println!("═══════════════════════════════════════");
        println!("1. 💬 AI Chat Assistant");
        println!("2. 🔧 Circuit View (Coming Soon)");
        println!("3. 🔍 Research Console (Coming Soon)");
        println!("═══════════════════════════════════════");
        println!("Commands: help, quit");
    }

    async fn chat_interface(&mut self) -> crate::OpenCircuitResult<()> {
        println!("\n💬 AI Chat Assistant - Circuit Design Expert");
        println!("Type 'back' to return to main menu, 'clear' to clear history\n");

        // Display chat history
        if !self.state.chat_messages.is_empty() {
            println!("📜 Chat History:");
            for message in &self.state.chat_messages {
                self.display_message(message);
            }
            println!();
        }

        loop {
            print!("You: ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let user_input = input.trim();

            match user_input {
                "back" => break,
                "clear" => {
                    self.state.chat_messages.clear();
                    self.chat_handler = ChatHandler::new(); // Reset chat handler
                    println!("Chat history cleared! 🧹");
                    continue;
                }
                "" => continue,
                _ => {
                    // Add user message
                    let user_message = ChatMessage {
                        id: uuid::Uuid::new_v4().to_string(),
                        content: user_input.to_string(),
                        is_user: true,
                        timestamp: Utc::now(),
                    };
                    self.state.chat_messages.push(user_message.clone());

                    // Get AI response
                    print!("🤖 AI: ");
                    io::stdout().flush()?;
                    
                    match self.chat_handler.process_message(user_input).await {
                        Ok(ai_message) => {
                            println!("{}", ai_message.content);
                            
                            // Add AI response to history (it's already added by the handler)
                            self.state.chat_messages.push(ai_message);
                        }
                        Err(e) => {
                            println!("Sorry, I encountered an error: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn display_message(&self, message: &ChatMessage) {
        let time = message.timestamp.format("%H:%M:%S");
        let prefix = if message.is_user { "You" } else { "🤖 AI" };
        println!("[{}] {}: {}", time, prefix, message.content);
    }

    fn circuit_view(&self) {
        println!("\n🔧 Circuit View");
        println!("This feature will be implemented in Phase 3: Circuit Generation & Simulation");
        println!("Coming soon: Visual circuit editor, component library, and simulation tools");
    }

    fn research_console(&self) {
        println!("\n🔍 Research Console");
        println!("This feature will be implemented in Phase 6: Advanced AI Features");
        println!("Coming soon: Component research, datasheet analysis, and design recommendations");
    }

    fn show_help(&self) {
        println!("\n📖 OpenCircuit Help");
        println!("═══════════════════════════════════════");
        println!("Available Commands:");
        println!("  1 or chat    - Open AI chat assistant");
        println!("  2 or circuit - View circuit editor (coming soon)");
        println!("  3 or research - Open research console (coming soon)");
        println!("  help         - Show this help message");
        println!("  quit/exit/q  - Exit the application");
        println!("\nIn Chat Mode:");
        println!("  back         - Return to main menu");
        println!("  clear        - Clear chat history");
        println!("═══════════════════════════════════════");
    }
}

pub fn run_app() -> crate::OpenCircuitResult<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let mut app = ConsoleApp::new();
        app.run().await
    })
}