//! XY Chart diagram support
//!
//! This module provides data structures for XY Chart diagrams.
//! XY charts show data on a 2D coordinate system with line and bar plots.

mod types;
pub mod parser;

pub use types::{
    AxisType, BandAxisData, ChartOrientation, DataPoint, LinearAxisData, Plot, PlotType,
    XAxisData, XYChartDb, YAxisData,
};
pub use parser::parse;
