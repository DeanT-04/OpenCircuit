use anyhow::Result;
use tracing::{info, warn};

// Re-export the crates for easy access
pub use opencircuit_ai as ai;
pub use opencircuit_circuit as circuit;
pub use opencircuit_core as core;
pub use opencircuit_database as database;
pub use opencircuit_gui as gui;
pub use opencircuit_pcb as pcb;
pub use opencircuit_utils as utils;

// Re-export commonly used types
pub use opencircuit_core::{OpenCircuitError, AppConfig, Project, Position, Size, Rect};
pub use opencircuit_ai::{AiService, AiConfig, AiResponse, AiModel};
pub use opencircuit_circuit::{Circuit, Component, ComponentType};
pub use opencircuit_database::{Database, ComponentRecord};
pub use opencircuit_gui::{OpenCircuitApp, AppState};
pub use opencircuit_pcb::{PcbDesign, ComponentPlacement, Trace};

/// OpenCircuit library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the OpenCircuit library
pub fn init() -> Result<()> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("OpenCircuit v{} initialized", VERSION);
    Ok(())
}

/// Initialize the OpenCircuit library (alias for Tauri compatibility)
pub fn initialize() -> Result<()> {
    init()
}

/// Core application configuration (legacy compatibility)
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub ai_api_key: Option<String>,
    pub log_level: String,
    pub data_dir: std::path::PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("opencircuit");

        Self {
            database_url: format!("sqlite:{}/opencircuit.db", data_dir.display()),
            ai_api_key: std::env::var("OPENAI_API_KEY").ok(),
            log_level: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            data_dir,
        }
    }
}

impl Config {
    /// Load configuration from environment variables and config files
    pub fn load() -> Result<Self> {
        let config = Self::default();
        
        // Ensure data directory exists
        std::fs::create_dir_all(&config.data_dir)?;
        
        // Warn if API key is not set
        if config.ai_api_key.is_none() {
            warn!("OPENAI_API_KEY environment variable not set - AI features will be limited");
        }
        
        info!("Configuration loaded successfully");
        Ok(config)
    }
}

/// Result type alias for OpenCircuit operations (legacy compatibility)
pub type OpenCircuitResult<T> = std::result::Result<T, OpenCircuitError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(!config.database_url.is_empty());
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}