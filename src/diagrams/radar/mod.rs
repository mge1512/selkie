//! Radar diagram support
//!
//! This module provides data structures for radar (spider/web) diagrams.
//! Radar diagrams show multivariate data plotted on axes radiating from a center point.

mod types;
pub mod parser;

pub use types::{Graticule, RadarAxis, RadarCurve, RadarDb, RadarEntry, RadarOptions};
pub use parser::parse;
