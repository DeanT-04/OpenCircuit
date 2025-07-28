//! Utility functions and shared code
//! 
//! This module contains:
//! - Common data structures
//! - Helper functions
//! - Constants and enumerations
//! - File I/O utilities

use crate::OpenCircuitResult;
use std::path::Path;

/// Application constants
pub mod constants {
    pub const APP_NAME: &str = "OpenCircuit";
    pub const CONFIG_FILE: &str = "config.toml";
    pub const DATABASE_FILE: &str = "opencircuit.db";
    pub const DEFAULT_WINDOW_WIDTH: f32 = 1200.0;
    pub const DEFAULT_WINDOW_HEIGHT: f32 = 800.0;
}

/// File format utilities
pub mod file_formats {
    use super::*;
    
    /// Supported export formats
    #[derive(Debug, Clone)]
    pub enum ExportFormat {
        KiCad,
        Eagle,
        Altium,
        Gerber,
        Pdf,
    }
    
    impl ExportFormat {
        pub fn extension(&self) -> &'static str {
            match self {
                ExportFormat::KiCad => ".kicad_pcb",
                ExportFormat::Eagle => ".brd",
                ExportFormat::Altium => ".PrjPCB",
                ExportFormat::Gerber => ".gbr",
                ExportFormat::Pdf => ".pdf",
            }
        }
    }
}

/// Validation utilities
pub mod validation {
    use super::*;
    
    /// Validate component part number format
    pub fn validate_part_number(part_number: &str) -> bool {
        !part_number.is_empty() && part_number.len() <= 50
    }
    
    /// Validate email format (for user accounts)
    pub fn validate_email(email: &str) -> bool {
        email.contains('@') && email.contains('.')
    }
    
    /// Validate file path
    pub fn validate_file_path(path: &Path) -> OpenCircuitResult<()> {
        if path.exists() {
            Ok(())
        } else {
            Err(crate::OpenCircuitError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", path.display()),
            )))
        }
    }
}

/// Math utilities for circuit calculations
pub mod math {
    /// Calculate parallel resistance
    pub fn parallel_resistance(r1: f64, r2: f64) -> f64 {
        if r1 == 0.0 || r2 == 0.0 {
            0.0
        } else {
            (r1 * r2) / (r1 + r2)
        }
    }
    
    /// Calculate series resistance
    pub fn series_resistance(resistances: &[f64]) -> f64 {
        resistances.iter().sum()
    }
    
    /// Convert degrees to radians
    pub fn deg_to_rad(degrees: f64) -> f64 {
        degrees * std::f64::consts::PI / 180.0
    }
    
    /// Convert radians to degrees
    pub fn rad_to_deg(radians: f64) -> f64 {
        radians * 180.0 / std::f64::consts::PI
    }
}

/// String utilities
pub mod string_utils {
    /// Sanitize filename for cross-platform compatibility
    pub fn sanitize_filename(filename: &str) -> String {
        filename
            .chars()
            .map(|c| match c {
                '<' | '>' | ':' | '"' | '|' | '?' | '*' => '_',
                '/' | '\\' => '_',
                c if c.is_control() => '_',
                c => c,
            })
            .collect()
    }
    
    /// Truncate string to specified length with ellipsis
    pub fn truncate_with_ellipsis(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else if max_len <= 3 {
            "...".to_string()
        } else {
            format!("{}...", &s[..max_len - 3])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_resistance() {
        assert_eq!(math::parallel_resistance(10.0, 10.0), 5.0);
        assert_eq!(math::parallel_resistance(0.0, 10.0), 0.0);
    }
    
    #[test]
    fn test_series_resistance() {
        assert_eq!(math::series_resistance(&[10.0, 20.0, 30.0]), 60.0);
        assert_eq!(math::series_resistance(&[]), 0.0);
    }
    
    #[test]
    fn test_angle_conversion() {
        assert!((math::deg_to_rad(180.0) - std::f64::consts::PI).abs() < 1e-10);
        assert!((math::rad_to_deg(std::f64::consts::PI) - 180.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_validation() {
        assert!(validation::validate_part_number("R1234"));
        assert!(!validation::validate_part_number(""));
        
        assert!(validation::validate_email("test@example.com"));
        assert!(!validation::validate_email("invalid-email"));
    }
    
    #[test]
    fn test_string_utils() {
        assert_eq!(string_utils::sanitize_filename("file<>name"), "file__name");
        assert_eq!(string_utils::truncate_with_ellipsis("hello world", 8), "hello...");
        assert_eq!(string_utils::truncate_with_ellipsis("short", 10), "short");
    }
    
    #[test]
    fn test_export_format() {
        assert_eq!(file_formats::ExportFormat::KiCad.extension(), ".kicad_pcb");
        assert_eq!(file_formats::ExportFormat::Gerber.extension(), ".gbr");
    }
}