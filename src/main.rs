use anyhow::Result;
use opencircuit::{init, Config};
use opencircuit::gui::OpenCircuitApp;
use tracing::{error, info};

fn main() -> Result<()> {
    // Initialize the library
    init()?;
    
    // Load configuration
    let config = Config::load()?;
    info!("Starting OpenCircuit v{}", opencircuit::VERSION);
    info!("Data directory: {}", config.data_dir.display());
    
    println!("🔌 Welcome to OpenCircuit!");
    println!("AI-powered circuit design and PCB layout tool");
    println!();
    println!("Launching GUI application...");
    
    // Launch the GUI application
    match OpenCircuitApp::run() {
        Ok(_) => {
            info!("GUI application closed successfully");
        }
        Err(e) => {
            error!("GUI application error: {}", e);
            eprintln!("Error running GUI: {}", e);
            return Err(e.into());
        }
    }
    
    info!("OpenCircuit session completed");
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