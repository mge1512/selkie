//! mermaid-rs - A Rust port of mermaid.js
//!
//! This library provides parsing and data structures for mermaid diagram syntax.
//! It supports multiple diagram types including flowcharts, sequence diagrams,
//! pie charts, and more.

pub mod common;
pub mod config;
pub mod diagrams;
pub mod error;

pub use config::Config;
pub use error::{MermaidError, Result};

/// Parse a mermaid diagram and return a diagram representation
pub fn parse(input: &str) -> Result<diagrams::Diagram> {
    let diagram_type = diagrams::detect_type(input)?;
    diagrams::parse(diagram_type, input)
}
