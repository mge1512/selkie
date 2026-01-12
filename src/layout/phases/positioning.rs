//! Phase 4: Node Positioning
//!
//! Assign x,y coordinates to all nodes based on their layer and order.
//! This is a simplified version of the Brandes-Kopf algorithm.

use crate::layout::graph::LayoutGraph;
use crate::layout::types::LayoutDirection;

/// Position all nodes in the graph
pub fn position_nodes(graph: &mut LayoutGraph) {
    let direction = graph.options.direction;
    let node_spacing = graph.options.node_spacing;
    // layer_spacing is accessed via graph.options in calculate_layer_info
    let padding = graph.options.padding;

    // Get layer information
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

    // Calculate layer sizes and positions
    let layer_info = calculate_layer_info(graph, num_layers, direction);

    // Position nodes within each layer
    for layer in 0..num_layers {
        position_layer(graph, layer, &layer_info, node_spacing, direction, &padding);
    }

    // Calculate graph bounds
    graph.compute_bounds();
}

/// Information about a single layer
#[derive(Debug, Clone)]
struct LayerInfo {
    /// Layer index
    #[allow(dead_code)]
    layer: usize,
    /// Maximum node size in the primary direction
    max_size: f64,
    /// Position of this layer (start coordinate)
    position: f64,
    /// Nodes in this layer, sorted by order
    nodes: Vec<String>,
}

/// Calculate layer dimensions and positions
fn calculate_layer_info(
    graph: &LayoutGraph,
    num_layers: usize,
    direction: LayoutDirection,
) -> Vec<LayerInfo> {
    let layer_spacing = graph.options.layer_spacing;
    let is_horizontal = direction.is_horizontal();
    let is_reversed = direction.is_reversed();

    // Collect nodes by layer
    let mut layers: Vec<LayerInfo> = (0..num_layers)
        .map(|l| LayerInfo {
            layer: l,
            max_size: 0.0,
            position: 0.0,
            nodes: Vec::new(),
        })
        .collect();

    // Populate layer info
    for node in &graph.nodes {
        if let Some(layer) = node.layer {
            if layer < num_layers {
                let size = if is_horizontal { node.width } else { node.height };
                layers[layer].max_size = layers[layer].max_size.max(size);
                layers[layer].nodes.push(node.id.clone());
            }
        }
    }

    // Sort nodes by order within each layer
    for layer_info in &mut layers {
        layer_info.nodes.sort_by_key(|id| {
            graph.get_node(id).and_then(|n| n.order).unwrap_or(0)
        });
    }

    // Calculate layer positions
    let padding = if is_horizontal {
        graph.options.padding.left
    } else {
        graph.options.padding.top
    };

    let mut position = padding;

    let layer_order: Vec<usize> = if is_reversed {
        (0..num_layers).rev().collect()
    } else {
        (0..num_layers).collect()
    };

    for &layer_idx in &layer_order {
        layers[layer_idx].position = position;
        position += layers[layer_idx].max_size + layer_spacing;
    }

    layers
}

/// Position nodes within a single layer
fn position_layer(
    graph: &mut LayoutGraph,
    layer: usize,
    layer_info: &[LayerInfo],
    node_spacing: f64,
    direction: LayoutDirection,
    padding: &crate::layout::types::Padding,
) {
    let info = &layer_info[layer];
    let is_horizontal = direction.is_horizontal();

    // Start position (left-aligned for now; centering could be added later)
    let cross_padding = if is_horizontal {
        padding.top
    } else {
        padding.left
    };
    let mut cross_position = cross_padding;

    // Position each node
    for node_id in &info.nodes {
        if let Some(node) = graph.get_node_mut(node_id) {
            if is_horizontal {
                // Horizontal layout: x is layer position, y is cross position
                node.x = Some(info.position);
                node.y = Some(cross_position);
                cross_position += node.height + node_spacing;
            } else {
                // Vertical layout: y is layer position, x is cross position
                node.x = Some(cross_position);
                node.y = Some(info.position);
                cross_position += node.width + node_spacing;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{LayoutEdge, LayoutNode, LayoutOptions};

    #[test]
    fn test_vertical_positioning() {
        let mut graph = LayoutGraph::new("test");
        graph.options = LayoutOptions {
            direction: LayoutDirection::TopToBottom,
            node_spacing: 20.0,
            layer_spacing: 40.0,
            ..Default::default()
        };

        let mut a = LayoutNode::new("A", 50.0, 30.0);
        a.layer = Some(0);
        a.order = Some(0);
        graph.add_node(a);

        let mut b = LayoutNode::new("B", 50.0, 30.0);
        b.layer = Some(1);
        b.order = Some(0);
        graph.add_node(b);

        position_nodes(&mut graph);

        let node_a = graph.get_node("A").unwrap();
        let node_b = graph.get_node("B").unwrap();

        // A should be above B (smaller y)
        assert!(node_a.y.unwrap() < node_b.y.unwrap());

        // They should be spaced by layer_spacing + max(height)
        let expected_gap = 30.0 + 40.0; // height + layer_spacing
        let actual_gap = node_b.y.unwrap() - node_a.y.unwrap();
        assert!((actual_gap - expected_gap).abs() < 1.0);
    }

    #[test]
    fn test_horizontal_positioning() {
        let mut graph = LayoutGraph::new("test");
        graph.options = LayoutOptions {
            direction: LayoutDirection::LeftToRight,
            node_spacing: 20.0,
            layer_spacing: 40.0,
            ..Default::default()
        };

        let mut a = LayoutNode::new("A", 50.0, 30.0);
        a.layer = Some(0);
        a.order = Some(0);
        graph.add_node(a);

        let mut b = LayoutNode::new("B", 50.0, 30.0);
        b.layer = Some(1);
        b.order = Some(0);
        graph.add_node(b);

        position_nodes(&mut graph);

        let node_a = graph.get_node("A").unwrap();
        let node_b = graph.get_node("B").unwrap();

        // A should be left of B (smaller x)
        assert!(node_a.x.unwrap() < node_b.x.unwrap());
    }

    #[test]
    fn test_multiple_nodes_per_layer() {
        let mut graph = LayoutGraph::new("test");
        graph.options = LayoutOptions {
            direction: LayoutDirection::TopToBottom,
            node_spacing: 20.0,
            layer_spacing: 40.0,
            ..Default::default()
        };

        // Two nodes in the same layer
        let mut a = LayoutNode::new("A", 50.0, 30.0);
        a.layer = Some(0);
        a.order = Some(0);
        graph.add_node(a);

        let mut b = LayoutNode::new("B", 50.0, 30.0);
        b.layer = Some(0);
        b.order = Some(1);
        graph.add_node(b);

        position_nodes(&mut graph);

        let node_a = graph.get_node("A").unwrap();
        let node_b = graph.get_node("B").unwrap();

        // Both should have the same y (same layer)
        assert_eq!(node_a.y, node_b.y);

        // B should be to the right of A (order 1 > order 0)
        assert!(node_b.x.unwrap() > node_a.x.unwrap());
    }
}
