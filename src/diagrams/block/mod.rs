//! Block diagram support
//!
//! This module provides data structures for block diagrams.
//! Block diagrams show blocks/nodes and their connections in a grid layout.

mod types;
pub mod parser;

pub use types::{Block, BlockDb, BlockType, ClassDef, Edge};
pub use parser::parse;
