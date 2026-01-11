//! Treemap diagram support
//!
//! This module provides data structures for treemap diagrams.
//! Treemap diagrams show hierarchical data as nested rectangles.

mod types;
pub mod parser;

pub use types::{build_hierarchy, StyleClass, TreemapDb, TreemapNode};
pub use parser::parse;
