//! Flowchart adapter for layout

use crate::diagrams::flowchart::{Direction, FlowchartDb, FlowVertexType};
use crate::error::Result;
use crate::layout::{
    LayoutDirection, LayoutEdge, LayoutGraph, LayoutNode, LayoutOptions,
    NodeShape, NodeSizeConfig, Padding, SizeEstimator, ToLayoutGraph,
};

impl ToLayoutGraph for FlowchartDb {
    fn to_layout_graph(&self, size_estimator: &dyn SizeEstimator) -> Result<LayoutGraph> {
        let config = NodeSizeConfig::default();
        let mut graph = LayoutGraph::new("flowchart");

        // Set layout options from diagram direction
        graph.options = LayoutOptions {
            direction: self.preferred_direction(),
            node_spacing: 50.0,
            layer_spacing: 50.0,
            padding: Padding::uniform(20.0),
        };

        // Convert vertices to layout nodes
        for (id, vertex) in self.vertices() {
            let shape = vertex
                .vertex_type
                .as_ref()
                .map(vertex_type_to_shape)
                .unwrap_or(NodeShape::Rectangle);

            let label = vertex.text.as_deref();
            let (width, height) = size_estimator.estimate_node_size(label, shape, &config);

            let mut node = LayoutNode::new(id, width, height)
                .with_shape(shape);

            if let Some(label) = label {
                node = node.with_label(label);
            }

            // Store original metadata
            node.metadata.insert("dom_id".to_string(), vertex.dom_id.clone());
            if let Some(vt) = &vertex.vertex_type {
                node.metadata.insert("vertex_type".to_string(), format!("{:?}", vt));
            }

            graph.add_node(node);
        }

        // Convert edges
        for edge in self.edges() {
            let edge_id = edge.id.clone().unwrap_or_else(|| {
                format!("{}-{}", edge.start, edge.end)
            });

            let mut layout_edge = LayoutEdge::new(&edge_id, &edge.start, &edge.end);

            if !edge.text.is_empty() {
                layout_edge = layout_edge.with_label(&edge.text);
            }

            // Set weight based on length hint
            if let Some(length) = edge.length {
                layout_edge = layout_edge.with_weight(length);
            }

            // Store edge type for rendering
            if let Some(et) = &edge.edge_type {
                layout_edge.metadata.insert("edge_type".to_string(), et.clone());
            }
            layout_edge.metadata.insert("stroke".to_string(), format!("{:?}", edge.stroke));

            graph.add_edge(layout_edge);
        }

        // Handle subgraphs as compound nodes
        // For now, we'll flatten them - proper nesting would require hierarchical layout
        // TODO: Implement proper subgraph nesting

        Ok(graph)
    }

    fn preferred_direction(&self) -> LayoutDirection {
        match Direction::parse(self.get_direction()) {
            Direction::TopToBottom => LayoutDirection::TopToBottom,
            Direction::BottomToTop => LayoutDirection::BottomToTop,
            Direction::LeftToRight => LayoutDirection::LeftToRight,
            Direction::RightToLeft => LayoutDirection::RightToLeft,
        }
    }
}

/// Convert FlowVertexType to NodeShape
fn vertex_type_to_shape(vt: &FlowVertexType) -> NodeShape {
    match vt {
        FlowVertexType::Square | FlowVertexType::Rect => NodeShape::Rectangle,
        FlowVertexType::Round => NodeShape::RoundedRect,
        FlowVertexType::Circle => NodeShape::Circle,
        FlowVertexType::DoubleCircle => NodeShape::DoubleCircle,
        FlowVertexType::Ellipse => NodeShape::Ellipse,
        FlowVertexType::Stadium => NodeShape::Stadium,
        FlowVertexType::Diamond => NodeShape::Diamond,
        FlowVertexType::Hexagon => NodeShape::Hexagon,
        FlowVertexType::Cylinder => NodeShape::Cylinder,
        FlowVertexType::Subroutine => NodeShape::Subroutine,
        FlowVertexType::Trapezoid => NodeShape::Trapezoid,
        FlowVertexType::InvTrapezoid => NodeShape::InvTrapezoid,
        FlowVertexType::LeanRight => NodeShape::LeanRight,
        FlowVertexType::LeanLeft => NodeShape::LeanLeft,
        FlowVertexType::Odd => NodeShape::Odd,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::CharacterSizeEstimator;

    #[test]
    fn test_simple_flowchart_to_layout() {
        let mut db = FlowchartDb::new();
        db.set_direction("LR");
        db.add_vertex_simple("A", Some("Start"), Some(FlowVertexType::Round));
        db.add_vertex_simple("B", Some("Process"), Some(FlowVertexType::Rect));
        db.add_vertex_simple("C", Some("Decision"), Some(FlowVertexType::Diamond));
        db.add_edge("A", "B", "-->", None, None);
        db.add_edge("B", "C", "-->", None, None);

        let estimator = CharacterSizeEstimator::default();
        let graph = db.to_layout_graph(&estimator).unwrap();

        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);
        assert_eq!(graph.options.direction, LayoutDirection::LeftToRight);

        // Check shapes
        let node_a = graph.get_node("A").unwrap();
        assert_eq!(node_a.shape, NodeShape::RoundedRect);

        let node_c = graph.get_node("C").unwrap();
        assert_eq!(node_c.shape, NodeShape::Diamond);
    }

    #[test]
    fn test_edge_labels() {
        let mut db = FlowchartDb::new();
        db.add_vertex_simple("A", Some("Start"), None);
        db.add_vertex_simple("B", Some("End"), None);
        db.add_edge("A", "B", "-->", Some("Yes"), None);

        let estimator = CharacterSizeEstimator::default();
        let graph = db.to_layout_graph(&estimator).unwrap();

        let edge = &graph.edges[0];
        assert_eq!(edge.label.as_deref(), Some("Yes"));
    }
}
