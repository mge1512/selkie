//! Quadrant chart diagram support
//!
//! This module provides data structures for quadrant chart diagrams.
//! Quadrant charts divide data into four quadrants based on two axes,
//! commonly used for prioritization matrices.

mod types;
pub mod parser;

pub use types::{ClassDef, PointStyle, QuadrantDb, QuadrantPoint};
pub use parser::parse;
