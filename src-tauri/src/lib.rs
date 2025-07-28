use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[tauri::command]
async fn initialize_opencircuit() -> Result<String, String> {
    // Initialize the OpenCircuit core library
    match opencircuit::initialize() {
        Ok(_) => Ok("OpenCircuit initialized successfully".to_string()),
        Err(e) => Err(format!("Failed to initialize OpenCircuit: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Setup logging
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Log application startup
            log::info!("OpenCircuit Tauri application starting...");
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_app_version,
            initialize_opencircuit
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
