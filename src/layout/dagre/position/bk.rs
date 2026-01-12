//! Brandes-Köpf x-coordinate assignment
//!
//! Based on "Fast and Simple Horizontal Coordinate Assignment" by Brandes and Köpf.
//!
//! This is a simplified implementation that focuses on getting reasonable x-coordinates.
//! The full algorithm uses four passes (ul, ur, dl, dr) and balances them.

use crate::layout::dagre::graph::DagreGraph;
use std::collections::HashMap;

/// Assign x coordinates to all nodes
pub fn position_x(g: &DagreGraph) -> HashMap<String, f64> {
    let nodesep = g.graph().nodesep;

    // Build layer matrix
    let max_rank = g.nodes()
        .iter()
        .filter_map(|v| g.node(v).and_then(|n| n.rank))
        .max()
        .unwrap_or(0) as usize;

    let mut layers: Vec<Vec<(String, usize)>> = (0..=max_rank).map(|_| Vec::new()).collect();

    for v in g.nodes() {
        if let Some(node) = g.node(v) {
            if let (Some(rank), Some(order)) = (node.rank, node.order) {
                if rank >= 0 && (rank as usize) <= max_rank {
                    layers[rank as usize].push((v.clone(), order));
                }
            }
        }
    }

    // Sort each layer by order
    for layer in &mut layers {
        layer.sort_by_key(|(_, order)| *order);
    }

    // Simple x-coordinate assignment: place nodes left to right with separation
    let mut xs: HashMap<String, f64> = HashMap::new();

    // First pass: assign initial x based on order within layer
    for layer in &layers {
        let mut x = 0.0;
        for (v, _) in layer {
            let width = g.node(v).map(|n| n.width).unwrap_or(0.0);
            xs.insert(v.clone(), x + width / 2.0);
            x += width + nodesep;
        }
    }

    // Find the maximum width of any layer
    let max_layer_width: f64 = layers.iter().map(|layer| {
        let mut width = 0.0;
        for (v, _) in layer {
            let node_width = g.node(v).map(|n| n.width).unwrap_or(0.0);
            width += node_width + nodesep;
        }
        width - nodesep // Remove last nodesep
    }).fold(0.0_f64, f64::max);

    // Center each layer
    for layer in &layers {
        let layer_width: f64 = {
            let mut width = 0.0;
            for (v, _) in layer {
                let node_width = g.node(v).map(|n| n.width).unwrap_or(0.0);
                width += node_width + nodesep;
            }
            width - nodesep
        };

        let offset = (max_layer_width - layer_width) / 2.0;

        for (v, _) in layer {
            if let Some(x) = xs.get_mut(v) {
                *x += offset;
            }
        }
    }

    // Apply barycenter adjustment for better alignment with edges
    // This is a simplified version - iterate a few times to improve positions
    for _ in 0..3 {
        // Down sweep: adjust x based on predecessors
        for rank in 1..=max_rank {
            for (v, _) in &layers[rank] {
                let in_edges = g.in_edges(v);
                if !in_edges.is_empty() {
                    let avg_x: f64 = in_edges.iter()
                        .filter_map(|e| xs.get(&e.v))
                        .copied()
                        .sum::<f64>() / in_edges.len() as f64;

                    // Move towards average of predecessors, but not too far
                    if let Some(x) = xs.get_mut(v) {
                        *x = (*x + avg_x) / 2.0;
                    }
                }
            }

            // Resolve overlaps in this layer
            resolve_overlaps(g, &layers[rank], &mut xs, nodesep);
        }

        // Up sweep: adjust x based on successors
        for rank in (0..max_rank).rev() {
            for (v, _) in &layers[rank] {
                let out_edges = g.out_edges(v);
                if !out_edges.is_empty() {
                    let avg_x: f64 = out_edges.iter()
                        .filter_map(|e| xs.get(&e.w))
                        .copied()
                        .sum::<f64>() / out_edges.len() as f64;

                    // Move towards average of successors, but not too far
                    if let Some(x) = xs.get_mut(v) {
                        *x = (*x + avg_x) / 2.0;
                    }
                }
            }

            // Resolve overlaps in this layer
            resolve_overlaps(g, &layers[rank], &mut xs, nodesep);
        }
    }

    xs
}

/// Resolve overlapping nodes in a layer
fn resolve_overlaps(
    g: &DagreGraph,
    layer: &[(String, usize)],
    xs: &mut HashMap<String, f64>,
    nodesep: f64,
) {
    if layer.len() < 2 {
        return;
    }

    // Sort by current x position
    let mut sorted: Vec<(&String, f64, f64)> = layer.iter()
        .filter_map(|(v, _)| {
            let x = xs.get(v)?;
            let width = g.node(v).map(|n| n.width).unwrap_or(0.0);
            Some((v, *x, width))
        })
        .collect();

    sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    // Resolve overlaps from left to right
    for i in 1..sorted.len() {
        let (prev_v, prev_x, prev_width) = sorted[i - 1];
        let (curr_v, curr_x, curr_width) = sorted[i];

        let min_x = prev_x + prev_width / 2.0 + nodesep + curr_width / 2.0;

        if curr_x < min_x {
            if let Some(x) = xs.get_mut(curr_v) {
                *x = min_x;
            }
            // Update sorted for next iteration
            sorted[i] = (curr_v, min_x, curr_width);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::dagre::graph::{NodeLabel, EdgeLabel};
    use crate::layout::dagre::rank;
    use crate::layout::dagre::order;
    use crate::layout::dagre::Ranker;

    #[test]
    fn test_position_x_single_node() {
        let mut g = DagreGraph::new();
        g.set_node("a", NodeLabel { width: 50.0, height: 100.0, ..Default::default() });
        rank::assign_ranks(&mut g, Ranker::LongestPath);
        order::order(&mut g);

        let xs = position_x(&g);

        assert!(xs.contains_key("a"));
        assert_eq!(xs["a"], 25.0); // width/2 = 50/2 = 25
    }

    #[test]
    fn test_position_x_two_nodes_same_rank() {
        let mut g = DagreGraph::new();
        g.graph_mut().nodesep = 50.0;
        g.set_node("a", NodeLabel { width: 50.0, height: 100.0, rank: Some(0), order: Some(0), ..Default::default() });
        g.set_node("b", NodeLabel { width: 50.0, height: 100.0, rank: Some(0), order: Some(1), ..Default::default() });

        let xs = position_x(&g);

        assert!(xs.contains_key("a"));
        assert!(xs.contains_key("b"));

        // b should be to the right of a
        assert!(xs["b"] > xs["a"]);

        // They should be separated by nodesep
        let sep = xs["b"] - xs["a"];
        assert!(sep >= 50.0 + 50.0); // width + nodesep at minimum
    }

    #[test]
    fn test_position_x_chain() {
        let mut g = DagreGraph::new();
        g.set_node("a", NodeLabel { width: 50.0, height: 40.0, ..Default::default() });
        g.set_node("b", NodeLabel { width: 50.0, height: 40.0, ..Default::default() });
        g.set_edge("a", "b", EdgeLabel::default());
        rank::assign_ranks(&mut g, Ranker::LongestPath);
        order::order(&mut g);

        let xs = position_x(&g);

        // Both should be centered (single node per layer)
        assert!(xs.contains_key("a"));
        assert!(xs.contains_key("b"));
        // They should be close to each other (vertically aligned)
        assert!((xs["a"] - xs["b"]).abs() < 30.0);
    }
}
