//! SVG rendering for mermaid diagrams

mod document;
mod edges;
mod elements;
mod markers;
mod shapes;
pub mod structure;
mod theme;

pub use document::SvgDocument;
pub use elements::{Attrs, SvgElement};
pub use structure::{CompareConfig, ComparisonResult, SvgStructure};
pub use theme::Theme;

use crate::diagrams::flowchart::FlowchartDb;
use crate::error::Result;
use crate::layout::LayoutGraph;

/// Configuration for SVG rendering
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Theme for colors and fonts
    pub theme: Theme,
    /// Padding around the diagram
    pub padding: f64,
    /// Include embedded CSS in SVG
    pub embed_css: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            padding: 20.0,
            embed_css: true,
        }
    }
}

/// SVG renderer for diagrams
#[derive(Debug, Clone)]
pub struct SvgRenderer {
    config: RenderConfig,
}

impl SvgRenderer {
    pub fn new(config: RenderConfig) -> Self {
        Self { config }
    }

    /// Render a flowchart to SVG
    pub fn render_flowchart(&self, db: &FlowchartDb, graph: &LayoutGraph) -> Result<String> {
        let mut doc = SvgDocument::new();

        // Set dimensions
        let width = graph.width.unwrap_or(800.0);
        let height = graph.height.unwrap_or(600.0);
        doc.set_size(width, height);

        // Add theme styles
        if self.config.embed_css {
            doc.add_style(&self.config.theme.generate_css());
        }

        // Add marker definitions
        doc.add_defs(markers::create_arrow_markers(&self.config.theme));

        // Render nodes
        for node in &graph.nodes {
            if node.is_dummy {
                continue;
            }

            // Get the original vertex info
            if let Some(vertex) = db.vertices().get(&node.id) {
                let shape_element = shapes::render_shape(node, vertex, &self.config.theme);
                doc.add_element(shape_element);
            }
        }

        // Render edges
        for edge in &graph.edges {
            // Skip dummy edges
            if edge.id.contains("_dummy_") {
                continue;
            }

            // Get the original edge info
            if let Some(flow_edge) = db.edges().iter().find(|e| {
                e.id.as_ref().map(|id| id == &edge.id).unwrap_or(false)
                    || (e.start == edge.sources.first().map(|s| s.as_str()).unwrap_or("")
                        && e.end == edge.targets.first().map(|s| s.as_str()).unwrap_or(""))
            }) {
                let edge_element = edges::render_edge(edge, flow_edge, &self.config.theme);
                doc.add_element(edge_element);
            }
        }

        Ok(doc.to_string())
    }
}
