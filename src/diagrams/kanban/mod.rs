//! Kanban diagram support
//!
//! This module provides data structures for kanban board diagrams.
//! Kanban diagrams show work items organized in columns/sections.

mod types;
pub mod parser;

pub use types::{KanbanData, KanbanDb, KanbanNode, NodeShape, Priority};
pub use parser::parse;
