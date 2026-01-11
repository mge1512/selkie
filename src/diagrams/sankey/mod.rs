//! Sankey diagram support
//!
//! This module provides data structures for Sankey diagrams.
//! Sankey diagrams show flow/movement between nodes with weighted connections.

mod types;
pub mod parser;

pub use types::{GraphLink, GraphNode, SankeyDb, SankeyGraph, SankeyLink, SankeyNode};
pub use parser::parse;
