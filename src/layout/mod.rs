//! Layout engine for positioning diagram elements
//!
//! This module provides a graph layout engine based on ELK's layered algorithm
//! (Sugiyama method). It takes diagram elements and computes their positions
//! for rendering.
//!
//! The `dagre` submodule provides a port of dagre.js for improved visual parity
//! with mermaid.js.

mod adapter;
mod graph;
mod size;
mod types;

pub mod dagre;
pub mod phases;

pub use adapter::{NodeSizeConfig, SizeEstimator, ToLayoutGraph};
pub use graph::LayoutGraph;
pub use size::CharacterSizeEstimator;
pub use types::{
    LayoutDirection, LayoutEdge, LayoutNode, LayoutOptions, NodeShape, Padding, Point,
};

use crate::error::Result;

/// Perform layout on a graph and return positioned nodes and routed edges
pub fn layout(mut graph: LayoutGraph) -> Result<LayoutGraph> {
    let mut engine = LayeredLayoutEngine::new();
    engine.run(&mut graph)?;
    Ok(graph)
}

/// The layered layout engine implementing Sugiyama's algorithm
pub struct LayeredLayoutEngine {
    /// Configuration for the layout phases
    _config: LayoutEngineConfig,
}

/// Configuration for the layout engine
#[derive(Debug, Clone)]
pub struct LayoutEngineConfig {
    /// Maximum iterations for crossing minimization
    pub max_iterations: usize,
}

impl Default for LayoutEngineConfig {
    fn default() -> Self {
        Self { max_iterations: 24 }
    }
}

impl LayeredLayoutEngine {
    /// Create a new layout engine with default configuration
    pub fn new() -> Self {
        Self {
            _config: LayoutEngineConfig::default(),
        }
    }

    /// Run the layout algorithm on the graph
    pub fn run(&mut self, graph: &mut LayoutGraph) -> Result<()> {
        // Phase 1: Cycle removal
        phases::remove_cycles(graph);

        // Phase 2: Layer assignment
        phases::assign_layers(graph);

        // Phase 3: Crossing minimization
        phases::minimize_crossings(graph);

        // Phase 4: Node positioning
        phases::position_nodes(graph);

        // Phase 5: Edge routing
        phases::route_edges(graph);

        Ok(())
    }
}

impl Default for LayeredLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}
