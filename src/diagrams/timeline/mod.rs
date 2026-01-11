//! Timeline diagram support
//!
//! This module provides data structures for timeline diagrams.
//! Timeline diagrams show events organized by time periods or sections.

mod types;
pub mod parser;

pub use types::{TimelineDb, TimelineTask};
pub use parser::parse;
