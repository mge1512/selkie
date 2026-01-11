//! Gantt diagram support
//!
//! This module provides parsing and data structures for Gantt chart diagrams.
//! Gantt charts show project schedules with tasks, durations, and dependencies.

mod types;
pub mod parser;

pub use types::{
    DisplayMode, Duration, DurationUnit, GanttDb, Task, TaskFlags, WeekendStart,
};
pub use parser::parse;
