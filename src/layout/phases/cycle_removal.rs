//! Phase 1: Cycle Removal
//!
//! Remove cycles from the graph to make it acyclic (DAG).
//! Uses greedy DFS to find back edges and reverses them.

use std::collections::{HashMap, HashSet};

use crate::layout::graph::LayoutGraph;

/// Remove cycles from the graph by reversing back edges
pub fn remove_cycles(graph: &mut LayoutGraph) {
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    let mut edges_to_reverse = Vec::new();

    // Build adjacency info
    let adj = graph.adjacency_list();
    let node_ids: Vec<String> = graph.all_node_ids().iter().map(|s| s.to_string()).collect();

    // DFS to find back edges
    for node_id in &node_ids {
        if !visited.contains(node_id.as_str()) {
            find_back_edges(
                node_id,
                &adj,
                &mut visited,
                &mut rec_stack,
                &mut edges_to_reverse,
            );
        }
    }

    // Reverse the back edges
    for (source, target) in edges_to_reverse {
        reverse_edge(graph, &source, &target);
    }
}

fn find_back_edges<'a>(
    node_id: &'a str,
    adj: &HashMap<&'a str, Vec<&'a str>>,
    visited: &mut HashSet<&'a str>,
    rec_stack: &mut HashSet<&'a str>,
    edges_to_reverse: &mut Vec<(String, String)>,
) {
    visited.insert(node_id);
    rec_stack.insert(node_id);

    if let Some(neighbors) = adj.get(node_id) {
        for &neighbor in neighbors {
            if !visited.contains(neighbor) {
                find_back_edges(neighbor, adj, visited, rec_stack, edges_to_reverse);
            } else if rec_stack.contains(neighbor) {
                // Found a back edge - mark for reversal
                edges_to_reverse.push((node_id.to_string(), neighbor.to_string()));
            }
        }
    }

    rec_stack.remove(node_id);
}

fn reverse_edge(graph: &mut LayoutGraph, source: &str, target: &str) {
    for edge in &mut graph.edges {
        if edge.sources.contains(&source.to_string())
            && edge.targets.contains(&target.to_string())
        {
            // Swap source and target
            std::mem::swap(&mut edge.sources, &mut edge.targets);
            edge.reversed = true;
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{LayoutEdge, LayoutNode};

    #[test]
    fn test_no_cycles() {
        let mut graph = LayoutGraph::new("test");
        graph.add_node(LayoutNode::new("A", 50.0, 30.0));
        graph.add_node(LayoutNode::new("B", 50.0, 30.0));
        graph.add_node(LayoutNode::new("C", 50.0, 30.0));
        graph.add_edge(LayoutEdge::new("e1", "A", "B"));
        graph.add_edge(LayoutEdge::new("e2", "B", "C"));

        assert!(!graph.has_cycles());
        remove_cycles(&mut graph);
        assert!(!graph.has_cycles());

        // No edges should be reversed
        assert!(graph.edges.iter().all(|e| !e.reversed));
    }

    #[test]
    fn test_simple_cycle() {
        let mut graph = LayoutGraph::new("test");
        graph.add_node(LayoutNode::new("A", 50.0, 30.0));
        graph.add_node(LayoutNode::new("B", 50.0, 30.0));
        graph.add_node(LayoutNode::new("C", 50.0, 30.0));
        graph.add_edge(LayoutEdge::new("e1", "A", "B"));
        graph.add_edge(LayoutEdge::new("e2", "B", "C"));
        graph.add_edge(LayoutEdge::new("e3", "C", "A"));

        assert!(graph.has_cycles());
        remove_cycles(&mut graph);
        assert!(!graph.has_cycles());

        // At least one edge should be reversed
        assert!(graph.edges.iter().any(|e| e.reversed));
    }

    #[test]
    fn test_self_loop() {
        let mut graph = LayoutGraph::new("test");
        graph.add_node(LayoutNode::new("A", 50.0, 30.0));
        graph.add_edge(LayoutEdge::new("e1", "A", "A"));

        assert!(graph.has_cycles());
        remove_cycles(&mut graph);
        // Self-loop should be reversed (becomes A->A still, but marked)
        assert!(graph.edges.iter().any(|e| e.reversed));
    }
}
