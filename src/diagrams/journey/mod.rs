//! User journey diagram support
//!
//! This module provides data structures for user journey diagrams.
//! User journey diagrams show user experiences as a series of tasks
//! with satisfaction scores and actors involved.

mod types;
pub mod parser;

pub use types::{JourneyDb, JourneyTask};
pub use parser::parse;
