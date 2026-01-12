//! Dagre layout algorithm - a port of dagre.js
//!
//! This module implements the dagre graph layout algorithm, which uses a layered
//! approach (Sugiyama method) with network simplex ranking and Brandes-Köpf
//! coordinate assignment.
//!
//! # Algorithm Phases
//!
//! The dagre algorithm consists of these main phases:
//! 1. **Cycle Removal** - Make the graph acyclic by reversing back edges
//! 2. **Rank Assignment** - Assign layers using network simplex
//! 3. **Ordering** - Minimize edge crossings within layers
//! 4. **Coordinate Assignment** - Assign x/y positions using Brandes-Köpf
//!
//! # References
//!
//! - Gansner et al. "A Technique for Drawing Directed Graphs" (1993)
//! - Brandes & Köpf "Fast and Simple Horizontal Coordinate Assignment" (2002)

pub mod acyclic;
pub mod graph;
pub mod order;
pub mod position;
pub mod rank;

use graph::DagreGraph;

/// Configuration for the dagre layout algorithm
#[derive(Debug, Clone)]
pub struct DagreConfig {
    /// Direction of the layout: TB, BT, LR, RL
    pub rankdir: RankDir,
    /// Separation between nodes on the same rank
    pub nodesep: f64,
    /// Separation between edges
    pub edgesep: f64,
    /// Separation between ranks
    pub ranksep: f64,
    /// Margin around the graph
    pub marginx: f64,
    pub marginy: f64,
    /// Method for breaking cycles: "greedy" or "dfs"
    pub acyclicer: Acyclicer,
    /// Method for ranking: "network-simplex", "tight-tree", or "longest-path"
    pub ranker: Ranker,
}

impl Default for DagreConfig {
    fn default() -> Self {
        Self {
            rankdir: RankDir::TB,
            nodesep: 50.0,
            edgesep: 20.0,
            ranksep: 50.0,
            marginx: 0.0,
            marginy: 0.0,
            acyclicer: Acyclicer::Greedy,
            ranker: Ranker::NetworkSimplex,
        }
    }
}

/// Direction for the rank layout
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankDir {
    /// Top to bottom
    TB,
    /// Bottom to top
    BT,
    /// Left to right
    LR,
    /// Right to left
    RL,
}

/// Method for breaking cycles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Acyclicer {
    /// Greedy feedback arc set (prefers low-weight edges)
    Greedy,
    /// DFS-based cycle breaking
    Dfs,
}

/// Method for assigning ranks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ranker {
    /// Network simplex algorithm (optimal)
    NetworkSimplex,
    /// Tight tree heuristic (fast)
    TightTree,
    /// Longest path (simple)
    LongestPath,
}

/// Run the dagre layout algorithm on a graph
pub fn layout(graph: &mut DagreGraph, config: &DagreConfig) {
    // Phase 1: Make the graph acyclic
    acyclic::run(graph, config.acyclicer);

    // Phase 2: Assign ranks to nodes
    rank::assign_ranks(graph, config.ranker);

    // Phase 3: Order nodes within ranks (crossing minimization)
    order::order(graph);

    // Phase 4: Assign coordinates
    position::position(graph);

    // Undo cycle removal (restore reversed edges)
    acyclic::undo(graph);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_single_node() {
        let mut g = DagreGraph::new();
        g.set_node("a", graph::NodeLabel { width: 50.0, height: 100.0, ..Default::default() });

        layout(&mut g, &DagreConfig::default());

        let node = g.node("a").unwrap();
        // x = width/2 = 25.0 (centered in single-node layer)
        assert_eq!(node.x, Some(25.0));
        // y = height/2 = 50.0 (centered vertically in its row)
        assert_eq!(node.y, Some(50.0));
    }

    #[test]
    fn test_layout_two_connected_nodes() {
        let mut g = DagreGraph::new();
        g.graph_mut().ranksep = 300.0;
        g.set_node("a", graph::NodeLabel { width: 50.0, height: 100.0, ..Default::default() });
        g.set_node("b", graph::NodeLabel { width: 75.0, height: 200.0, ..Default::default() });
        g.set_edge("a", "b", graph::EdgeLabel::default());

        layout(&mut g, &DagreConfig { ranksep: 300.0, ..Default::default() });

        let a = g.node("a").unwrap();
        let b = g.node("b").unwrap();

        // Both should have x coordinates assigned
        assert!(a.x.is_some());
        assert!(b.x.is_some());

        // y coordinates should be correct
        // a.y = 100/2 = 50
        assert_eq!(a.y, Some(50.0));
        // b.y = 100 + 300 + 200/2 = 500
        assert_eq!(b.y, Some(500.0));
    }
}
