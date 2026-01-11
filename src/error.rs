//! Error types for mermaid-rs

use thiserror::Error;

/// Main error type for mermaid-rs operations
#[derive(Error, Debug)]
pub enum MermaidError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Unknown diagram type: {0}")]
    UnknownDiagramType(String),

    #[error("Invalid value: {message}")]
    InvalidValue { message: String },

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Result type alias for mermaid operations
pub type Result<T> = std::result::Result<T, MermaidError>;
