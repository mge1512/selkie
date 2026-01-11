//! Diagram types and parsing

pub mod flowchart;
pub mod pie;

mod detect;

pub use detect::{detect_type, DiagramType};

use crate::error::Result;

/// A parsed mermaid diagram
#[derive(Debug, Clone)]
pub enum Diagram {
    Flowchart(flowchart::FlowchartDb),
    Pie(pie::PieDb),
}

/// Parse a diagram of a specific type
pub fn parse(diagram_type: DiagramType, input: &str) -> Result<Diagram> {
    match diagram_type {
        DiagramType::Flowchart => {
            let db = flowchart::parse(input)?;
            Ok(Diagram::Flowchart(db))
        }
        DiagramType::Pie => {
            let db = pie::parse(input)?;
            Ok(Diagram::Pie(db))
        }
    }
}
