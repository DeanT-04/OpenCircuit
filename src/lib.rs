use anyhow::Result;
use tracing::{info, warn};

mod ai;
mod circuit;
mod database;
pub mod gui;
mod pcb;
mod utils;

pub use ai::*;
pub use circuit::*;
pub use database::*;
pub use gui::*;
pub use pcb::*;
pub use utils::*;

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

/// Core application configuration
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

/// Application error types
#[derive(Debug, thiserror::Error)]
pub enum OpenCircuitError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("AI service error: {0}")]
    Ai(String),
    
    #[error("Circuit simulation error: {0}")]
    Circuit(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("GUI error: {0}")]
    Gui(String),
    
    #[error("PCB design error: {0}")]
    Pcb(String),
}

/// Result type alias for OpenCircuit operations
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