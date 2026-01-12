//! Phase 3: Crossing Minimization
//!
//! Reorder nodes within layers to minimize edge crossings.
//! Uses the barycenter heuristic with layer sweep.

use std::collections::HashMap;

use crate::layout::graph::LayoutGraph;

/// Minimize edge crossings using barycenter method
pub fn minimize_crossings(graph: &mut LayoutGraph) {
    let max_iterations = 24;

    // Get initial layer structure
    let num_layers = graph
        .nodes
        .iter()
        .filter_map(|n| n.layer)
        .max()
        .map(|l| l + 1)
        .unwrap_or(0);

    if num_layers == 0 {
        return;
    }

    // Initialize order within each layer
    initialize_order(graph);

    // Perform sweep iterations
    for iteration in 0..max_iterations {
        let prev_crossings = count_crossings(graph);

        if iteration % 2 == 0 {
            // Sweep down (layer 0 to n-1)
            for layer in 1..num_layers {
                order_layer_by_barycenter(graph, layer, true);
            }
        } else {
            // Sweep up (layer n-1 to 0)
            for layer in (0..num_layers - 1).rev() {
                order_layer_by_barycenter(graph, layer, false);
            }
        }

        let new_crossings = count_crossings(graph);

        // Stop if no improvement
        if new_crossings >= prev_crossings && iteration > 4 {
            break;
        }
    }
}

/// Initialize order for nodes without an explicit order
fn initialize_order(graph: &mut LayoutGraph) {
    // Group nodes by layer
    let mut layers: HashMap<usize, Vec<String>> = HashMap::new();

    for node in &graph.nodes {
        if let Some(layer) = node.layer {
            layers
                .entry(layer)
                .or_default()
                .push(node.id.clone());
        }
    }

    // Assign order within each layer
    for (_, node_ids) in layers {
        for (order, id) in node_ids.iter().enumerate() {
            if let Some(node) = graph.get_node_mut(id) {
                if node.order.is_none() {
                    node.order = Some(order);
                }
            }
        }
    }
}

/// Reorder nodes in a layer based on barycenter of connected nodes
fn order_layer_by_barycenter(graph: &mut LayoutGraph, layer: usize, use_predecessors: bool) {
    // Collect nodes in this layer
    let nodes_in_layer: Vec<String> = graph
        .nodes
        .iter()
        .filter(|n| n.layer == Some(layer))
        .map(|n| n.id.clone())
        .collect();

    if nodes_in_layer.is_empty() {
        return;
    }

    // Calculate barycenter for each node
    let mut barycenters: Vec<(String, f64)> = Vec::new();

    for node_id in &nodes_in_layer {
        let connected_nodes = if use_predecessors {
            graph.predecessors(node_id)
        } else {
            graph.successors(node_id)
        };

        if connected_nodes.is_empty() {
            // Keep current order if no connections
            let current_order = graph
                .get_node(node_id)
                .and_then(|n| n.order)
                .unwrap_or(0);
            barycenters.push((node_id.clone(), current_order as f64));
        } else {
            // Calculate average order of connected nodes
            let sum: f64 = connected_nodes
                .iter()
                .filter_map(|id| graph.get_node(id).and_then(|n| n.order))
                .map(|o| o as f64)
                .sum();
            let count = connected_nodes
                .iter()
                .filter(|id| graph.get_node(id).and_then(|n| n.order).is_some())
                .count();

            if count > 0 {
                barycenters.push((node_id.clone(), sum / count as f64));
            } else {
                let current_order = graph
                    .get_node(node_id)
                    .and_then(|n| n.order)
                    .unwrap_or(0);
                barycenters.push((node_id.clone(), current_order as f64));
            }
        }
    }

    // Sort by barycenter
    barycenters.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // Apply new order
    for (order, (node_id, _)) in barycenters.iter().enumerate() {
        if let Some(node) = graph.get_node_mut(node_id) {
            node.order = Some(order);
        }
    }
}

/// Count the number of edge crossings in the graph
fn count_crossings(graph: &LayoutGraph) -> usize {
    let mut crossings = 0;

    // Get max layer
    let num_layers = graph
        .nodes
        .iter()
        .filter_map(|n| n.layer)
        .max()
        .map(|l| l + 1)
        .unwrap_or(0);

    // Count crossings between adjacent layers
    for layer in 0..num_layers.saturating_sub(1) {
        crossings += count_crossings_between_layers(graph, layer, layer + 1);
    }

    crossings
}

/// Count crossings between two adjacent layers
fn count_crossings_between_layers(graph: &LayoutGraph, layer1: usize, layer2: usize) -> usize {
    // Collect edges between the two layers with positions
    let mut edges: Vec<(usize, usize)> = Vec::new();

    for edge in &graph.edges {
        if let (Some(source), Some(target)) = (edge.source(), edge.target()) {
            let source_node = graph.get_node(source);
            let target_node = graph.get_node(target);

            if let (Some(sn), Some(tn)) = (source_node, target_node) {
                if sn.layer == Some(layer1) && tn.layer == Some(layer2) {
                    if let (Some(s_order), Some(t_order)) = (sn.order, tn.order) {
                        edges.push((s_order, t_order));
                    }
                } else if sn.layer == Some(layer2) && tn.layer == Some(layer1) {
                    if let (Some(s_order), Some(t_order)) = (sn.order, tn.order) {
                        edges.push((t_order, s_order));
                    }
                }
            }
        }
    }

    // Count crossings: two edges (a1, b1) and (a2, b2) cross if
    // (a1 < a2 && b1 > b2) || (a1 > a2 && b1 < b2)
    let mut crossings = 0;
    for i in 0..edges.len() {
        for j in (i + 1)..edges.len() {
            let (a1, b1) = edges[i];
            let (a2, b2) = edges[j];
            if (a1 < a2 && b1 > b2) || (a1 > a2 && b1 < b2) {
                crossings += 1;
            }
        }
    }

    crossings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{LayoutEdge, LayoutNode};

    #[test]
    fn test_crossing_count() {
        let mut graph = LayoutGraph::new("test");

        // Create a crossing: A->D and B->C where A,B are layer 0 and C,D are layer 1
        let mut a = LayoutNode::new("A", 50.0, 30.0);
        a.layer = Some(0);
        a.order = Some(0);
        graph.add_node(a);

        let mut b = LayoutNode::new("B", 50.0, 30.0);
        b.layer = Some(0);
        b.order = Some(1);
        graph.add_node(b);

        let mut c = LayoutNode::new("C", 50.0, 30.0);
        c.layer = Some(1);
        c.order = Some(0);
        graph.add_node(c);

        let mut d = LayoutNode::new("D", 50.0, 30.0);
        d.layer = Some(1);
        d.order = Some(1);
        graph.add_node(d);

        // A(0)->D(1) and B(1)->C(0) creates a crossing
        graph.add_edge(LayoutEdge::new("e1", "A", "D"));
        graph.add_edge(LayoutEdge::new("e2", "B", "C"));

        let crossings = count_crossings(&graph);
        assert_eq!(crossings, 1);
    }

    #[test]
    fn test_no_crossings() {
        let mut graph = LayoutGraph::new("test");

        let mut a = LayoutNode::new("A", 50.0, 30.0);
        a.layer = Some(0);
        a.order = Some(0);
        graph.add_node(a);

        let mut b = LayoutNode::new("B", 50.0, 30.0);
        b.layer = Some(0);
        b.order = Some(1);
        graph.add_node(b);

        let mut c = LayoutNode::new("C", 50.0, 30.0);
        c.layer = Some(1);
        c.order = Some(0);
        graph.add_node(c);

        let mut d = LayoutNode::new("D", 50.0, 30.0);
        d.layer = Some(1);
        d.order = Some(1);
        graph.add_node(d);

        // Parallel edges: A->C and B->D (no crossing)
        graph.add_edge(LayoutEdge::new("e1", "A", "C"));
        graph.add_edge(LayoutEdge::new("e2", "B", "D"));

        let crossings = count_crossings(&graph);
        assert_eq!(crossings, 0);
    }
}
