//! Diagram types and parsing

pub mod class;
pub mod flowchart;
pub mod git;
pub mod info;
pub mod mindmap;
pub mod pie;
pub mod sequence;

mod detect;

pub use detect::{detect_type, DiagramType};

use crate::error::Result;

/// A parsed mermaid diagram
#[derive(Debug, Clone)]
pub enum Diagram {
    Flowchart(flowchart::FlowchartDb),
    Info(info::InfoDb),
    Mindmap(mindmap::MindmapDb),
    Pie(pie::PieDb),
}

/// Parse a diagram of a specific type
pub fn parse(diagram_type: DiagramType, input: &str) -> Result<Diagram> {
    match diagram_type {
        DiagramType::Flowchart => {
            let db = flowchart::parse(input)?;
            Ok(Diagram::Flowchart(db))
        }
        DiagramType::Info => {
            let db = info::parse(input)?;
            Ok(Diagram::Info(db))
        }
        DiagramType::Mindmap => {
            let db = mindmap::parse(input)?;
            Ok(Diagram::Mindmap(db))
        }
        DiagramType::Pie => {
            let db = pie::parse(input)?;
            Ok(Diagram::Pie(db))
        }
    }
}
