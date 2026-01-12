//! Phase 2: Layer Assignment
//!
//! Assign nodes to layers (ranks) using the longest path algorithm.
//! This is simpler than network simplex but produces reasonable results.

use std::collections::{HashMap, HashSet};

use crate::layout::graph::LayoutGraph;

/// Assign layers to all nodes using longest path from sources
pub fn assign_layers(graph: &mut LayoutGraph) {
    // Find nodes with no incoming edges (sources)
    let mut in_degree: HashMap<&str, usize> = HashMap::new();

    for id in graph.all_node_ids() {
        in_degree.insert(id, graph.in_edges(id).len());
    }

    // Assign layers using longest path
    let mut layers: HashMap<String, usize> = HashMap::new();
    let mut _visited: HashSet<String> = HashSet::new();

    // Process nodes in topological order
    let topo_order = topological_sort(graph);

    for node_id in &topo_order {
        let predecessors = graph.predecessors(node_id);
        let layer = if predecessors.is_empty() {
            0
        } else {
            predecessors
                .iter()
                .filter_map(|p| layers.get(*p))
                .max()
                .map(|l| l + 1)
                .unwrap_or(0)
        };
        layers.insert(node_id.clone(), layer);
    }

    // Apply layers to nodes
    for (node_id, layer) in layers {
        if let Some(node) = graph.get_node_mut(&node_id) {
            node.layer = Some(layer);
        }
    }

    // Insert dummy nodes for long edges (spanning multiple layers)
    insert_dummy_nodes(graph);
}

/// Topological sort using Kahn's algorithm
fn topological_sort(graph: &LayoutGraph) -> Vec<String> {
    let all_ids: Vec<String> = graph.all_node_ids().iter().map(|s| s.to_string()).collect();
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();

    // Initialize
    for id in &all_ids {
        in_degree.insert(id.clone(), 0);
        adj.insert(id.clone(), Vec::new());
    }

    // Build in-degree and adjacency
    for edge in &graph.edges {
        for source in &edge.sources {
            for target in &edge.targets {
                adj.get_mut(source).map(|v| v.push(target.clone()));
                *in_degree.get_mut(target).unwrap() += 1;
            }
        }
    }

    // Find all nodes with in-degree 0
    let mut queue: Vec<String> = in_degree
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(id, _)| id.clone())
        .collect();

    let mut result = Vec::new();

    while let Some(node) = queue.pop() {
        result.push(node.clone());

        if let Some(neighbors) = adj.get(&node) {
            for neighbor in neighbors {
                if let Some(degree) = in_degree.get_mut(neighbor) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push(neighbor.clone());
                    }
                }
            }
        }
    }

    result
}

/// Insert dummy nodes for edges that span multiple layers
fn insert_dummy_nodes(graph: &mut LayoutGraph) {
    let mut dummy_counter = 0;
    let mut edges_to_modify: Vec<(usize, Vec<String>)> = Vec::new();
    let mut new_nodes: Vec<crate::layout::LayoutNode> = Vec::new();

    // Collect layer info
    let layer_map: HashMap<String, usize> = graph
        .nodes
        .iter()
        .filter_map(|n| n.layer.map(|l| (n.id.clone(), l)))
        .collect();

    // Find edges that span multiple layers
    for (edge_idx, edge) in graph.edges.iter().enumerate() {
        if let (Some(source), Some(target)) = (edge.source(), edge.target()) {
            if let (Some(&source_layer), Some(&target_layer)) =
                (layer_map.get(source), layer_map.get(target))
            {
                let span = if target_layer > source_layer {
                    target_layer - source_layer
                } else {
                    0
                };

                if span > 1 {
                    // Need dummy nodes
                    let mut chain = vec![source.to_string()];

                    for i in 1..span {
                        let dummy_id = format!("_dummy_{}_{}", edge.id, dummy_counter);
                        dummy_counter += 1;

                        let mut dummy = crate::layout::LayoutNode::dummy(&dummy_id, &edge.id);
                        dummy.layer = Some(source_layer + i);
                        new_nodes.push(dummy);
                        chain.push(dummy_id);
                    }

                    chain.push(target.to_string());
                    edges_to_modify.push((edge_idx, chain));
                }
            }
        }
    }

    // Add dummy nodes
    graph.nodes.extend(new_nodes);

    // Modify edges to use dummy nodes
    for (edge_idx, chain) in edges_to_modify.into_iter().rev() {
        let original_edge = &graph.edges[edge_idx];
        let edge_id = original_edge.id.clone();
        let label = original_edge.label.clone();
        let metadata = original_edge.metadata.clone();

        // Remove the original edge
        graph.edges.remove(edge_idx);

        // Add new edges through the chain
        for i in 0..chain.len() - 1 {
            let new_id = format!("{}_{}", edge_id, i);
            let mut new_edge =
                crate::layout::LayoutEdge::new(&new_id, &chain[i], &chain[i + 1]);

            // Put label on the middle edge
            if i == chain.len() / 2 - 1 {
                new_edge.label = label.clone();
            }

            new_edge.metadata = metadata.clone();
            graph.edges.push(new_edge);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{LayoutEdge, LayoutNode};

    #[test]
    fn test_simple_layering() {
        let mut graph = LayoutGraph::new("test");
        graph.add_node(LayoutNode::new("A", 50.0, 30.0));
        graph.add_node(LayoutNode::new("B", 50.0, 30.0));
        graph.add_node(LayoutNode::new("C", 50.0, 30.0));
        graph.add_edge(LayoutEdge::new("e1", "A", "B"));
        graph.add_edge(LayoutEdge::new("e2", "B", "C"));

        assign_layers(&mut graph);

        assert_eq!(graph.get_node("A").unwrap().layer, Some(0));
        assert_eq!(graph.get_node("B").unwrap().layer, Some(1));
        assert_eq!(graph.get_node("C").unwrap().layer, Some(2));
    }

    #[test]
    fn test_diamond_layering() {
        let mut graph = LayoutGraph::new("test");
        graph.add_node(LayoutNode::new("A", 50.0, 30.0));
        graph.add_node(LayoutNode::new("B", 50.0, 30.0));
        graph.add_node(LayoutNode::new("C", 50.0, 30.0));
        graph.add_node(LayoutNode::new("D", 50.0, 30.0));
        graph.add_edge(LayoutEdge::new("e1", "A", "B"));
        graph.add_edge(LayoutEdge::new("e2", "A", "C"));
        graph.add_edge(LayoutEdge::new("e3", "B", "D"));
        graph.add_edge(LayoutEdge::new("e4", "C", "D"));

        assign_layers(&mut graph);

        // A should be layer 0
        assert_eq!(graph.get_node("A").unwrap().layer, Some(0));
        // B and C should be layer 1
        assert_eq!(graph.get_node("B").unwrap().layer, Some(1));
        assert_eq!(graph.get_node("C").unwrap().layer, Some(1));
        // D should be layer 2
        assert_eq!(graph.get_node("D").unwrap().layer, Some(2));
    }
}
