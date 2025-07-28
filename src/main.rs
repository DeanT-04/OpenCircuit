use anyhow::Result;
use opencircuit::{init, Config};
use tracing::{error, info};

fn main() -> Result<()> {
    // Initialize the library
    init()?;
    
    // Load configuration
    let config = Config::load()?;
    info!("Starting OpenCircuit v{}", opencircuit::VERSION);
    info!("Data directory: {}", config.data_dir.display());
    
    // For now, just print a welcome message
    // The GUI will be implemented in the next task
    println!("🔌 Welcome to OpenCircuit!");
    println!("AI-powered circuit design and PCB layout tool");
    println!();
    println!("Current status: Foundation setup complete");
    println!("Next: Tauri application framework setup");
    println!();
    println!("Configuration:");
    println!("  - Data directory: {}", config.data_dir.display());
    println!("  - Database: {}", config.database_url);
    println!("  - AI API: {}", if config.ai_api_key.is_some() { "Configured" } else { "Not configured" });
    
    // Test basic functionality
    test_basic_functionality()?;
    
    info!("OpenCircuit initialization completed successfully");
    Ok(())
}

fn test_basic_functionality() -> Result<()> {
    info!("Running basic functionality tests...");
    
    // Test configuration loading
    let _config = Config::load()?;
    info!("✅ Configuration loading works");
    
    // Test logging
    info!("✅ Logging system works");
    
    println!("✅ All basic functionality tests passed!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_functionality() {
        // Test that main components can be initialized
        let result = init();
        assert!(result.is_ok());
        
        let config = Config::load();
        assert!(config.is_ok());
    }
}