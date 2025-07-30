//! Simulation error types and result handling

use thiserror::Error;

/// Result type for simulation operations
pub type Result<T> = std::result::Result<T, SimulationError>;

/// Simulation error types
#[derive(Error, Debug)]
pub enum SimulationError {
    #[error("NgSpice not found: {0}")]
    NgSpiceNotFound(String),
    
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Command failed: {command} - {error}")]
    CommandFailed {
        command: String,
        error: String,
    },
    
    #[error("Invalid component '{component}': {reason}")]
    InvalidComponent {
        component: String,
        reason: String,
    },
    
    #[error("Unsupported component type: {component_type}")]
    UnsupportedComponent {
        component_type: String,
    },
    
    #[error("Parse error in line '{line}': {reason}")]
    ParseError {
        line: String,
        reason: String,
    },
    
    #[error("Convergence failed: {reason}")]
    ConvergenceFailed {
        reason: String,
    },
    
    #[error("Memory allocation failed: {reason}")]
    MemoryError {
        reason: String,
    },
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("FFI error: {0}")]
    FfiError(String),
    
    #[error("Timeout: operation took longer than {timeout_ms}ms")]
    Timeout {
        timeout_ms: u64,
    },
    
    #[error("Analysis error: {analysis_type} - {reason}")]
    AnalysisError {
        analysis_type: String,
        reason: String,
    },
    
    #[error("Library loading error: {0}")]
    LibraryError(#[from] libloading::Error),
    
    #[error("Generic error: {0}")]
    Generic(#[from] anyhow::Error),
}

impl SimulationError {
    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            SimulationError::NgSpiceNotFound(_) => false,
            SimulationError::InitializationFailed(_) => false,
            SimulationError::LibraryError(_) => false,
            SimulationError::CommandFailed { .. } => true,
            SimulationError::ConvergenceFailed { .. } => true,
            SimulationError::Timeout { .. } => true,
            SimulationError::AnalysisError { .. } => true,
            _ => false,
        }
    }
    
    /// Get error category for logging and metrics
    pub fn category(&self) -> &'static str {
        match self {
            SimulationError::NgSpiceNotFound(_) => "setup",
            SimulationError::InitializationFailed(_) => "setup",
            SimulationError::LibraryError(_) => "setup",
            SimulationError::CommandFailed { .. } => "execution",
            SimulationError::InvalidComponent { .. } => "validation",
            SimulationError::UnsupportedComponent { .. } => "validation",
            SimulationError::ParseError { .. } => "parsing",
            SimulationError::ConvergenceFailed { .. } => "numerical",
            SimulationError::MemoryError { .. } => "system",
            SimulationError::IoError(_) => "io",
            SimulationError::FfiError(_) => "ffi",
            SimulationError::Timeout { .. } => "performance",
            SimulationError::AnalysisError { .. } => "analysis",
            SimulationError::Generic(_) => "unknown",
        }
    }
}