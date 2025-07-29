use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fmt;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub mod models;
pub mod apis;

pub use models::*;
pub use apis::*;

/// Core error types for the OpenCircuit application
#[derive(thiserror::Error, Debug)]
pub enum OpenCircuitError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("AI service error: {0}")]
    AiService(String),
    
    #[error("Circuit error: {0}")]
    Circuit(String),
    
    #[error("PCB error: {0}")]
    Pcb(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database_path: Option<String>,
    pub ai_service_url: String,
    pub ai_model: String,
    pub log_level: String,
    pub auto_save: bool,
    pub backup_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database_path: None,
            ai_service_url: "http://localhost:11434".to_string(),
            ai_model: "llama2".to_string(),
            log_level: "info".to_string(),
            auto_save: true,
            backup_enabled: true,
        }
    }
}

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub author: Option<String>,
}

impl Project {
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            created_at: now,
            updated_at: now,
            version: "1.0.0".to_string(),
            author: None,
        }
    }
    
    pub fn update(&mut self) {
        self.updated_at = chrono::Utc::now();
    }
}

/// Position in 2D space
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    
    pub fn distance_to(&self, other: &Position) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.2}, {:.2})", self.x, self.y)
    }
}

/// Size in 2D space
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
    
    pub fn area(&self) -> f64 {
        self.width * self.height
    }
}

/// Rectangle defined by position and size
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub position: Position,
    pub size: Size,
}

impl Rect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            position: Position::new(x, y),
            size: Size::new(width, height),
        }
    }
    
    pub fn contains(&self, point: &Position) -> bool {
        point.x >= self.position.x
            && point.x <= self.position.x + self.size.width
            && point.y >= self.position.y
            && point.y <= self.position.y + self.size.height
    }
    
    pub fn intersects(&self, other: &Rect) -> bool {
        !(self.position.x + self.size.width < other.position.x
            || other.position.x + other.size.width < self.position.x
            || self.position.y + self.size.height < other.position.y
            || other.position.y + other.size.height < self.position.y)
    }
}

/// Load application configuration
pub fn load_config() -> Result<AppConfig> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| OpenCircuitError::Config("Could not determine config directory".to_string()))?
        .join("OpenCircuit");
    
    std::fs::create_dir_all(&config_dir)?;
    let config_path = config_dir.join("config.toml");
    
    if config_path.exists() {
        let config_str = std::fs::read_to_string(&config_path)?;
        let config: AppConfig = toml::from_str(&config_str)
            .map_err(|e| OpenCircuitError::Config(format!("Failed to parse config: {}", e)))?;
        Ok(config)
    } else {
        let default_config = AppConfig::default();
        save_config(&default_config)?;
        Ok(default_config)
    }
}

/// Save application configuration
pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| OpenCircuitError::Config("Could not determine config directory".to_string()))?
        .join("OpenCircuit");
    
    std::fs::create_dir_all(&config_dir)?;
    let config_path = config_dir.join("config.toml");
    
    let config_str = toml::to_string_pretty(config)
        .map_err(|e| OpenCircuitError::Config(format!("Failed to serialize config: {}", e)))?;
    
    std::fs::write(&config_path, config_str)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_distance() {
        let p1 = Position::new(0.0, 0.0);
        let p2 = Position::new(3.0, 4.0);
        assert_eq!(p1.distance_to(&p2), 5.0);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        assert!(rect.contains(&Position::new(5.0, 5.0)));
        assert!(!rect.contains(&Position::new(15.0, 5.0)));
    }

    #[test]
    fn test_rect_intersects() {
        let rect1 = Rect::new(0.0, 0.0, 10.0, 10.0);
        let rect2 = Rect::new(5.0, 5.0, 10.0, 10.0);
        let rect3 = Rect::new(20.0, 20.0, 10.0, 10.0);
        
        assert!(rect1.intersects(&rect2));
        assert!(!rect1.intersects(&rect3));
    }

    #[test]
    fn test_project_creation() {
        let project = Project::new("Test Project".to_string());
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.version, "1.0.0");
    }

    #[test]
    fn test_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.ai_service_url, "http://localhost:11434");
        assert_eq!(config.ai_model, "llama2");
        assert!(config.auto_save);
    }
}