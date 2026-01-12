//! Phase 5: Edge Routing
//!
//! Compute edge paths with bend points for orthogonal routing.

use crate::layout::graph::LayoutGraph;
use crate::layout::types::{LayoutDirection, Point};

/// Routing info for a single edge
struct EdgeRoutingInfo {
    edge_idx: usize,
    bend_points: Vec<Point>,
    label_position: Option<Point>,
}

/// Route all edges in the graph
pub fn route_edges(graph: &mut LayoutGraph) {
    let direction = graph.options.direction;

    // First pass: calculate routing for all edges (immutable borrow)
    let routing_info: Vec<EdgeRoutingInfo> = graph
        .edges
        .iter()
        .enumerate()
        .filter_map(|(idx, edge)| {
            let source_id = edge.source()?;
            let target_id = edge.target()?;

            let source_node = graph.get_node(source_id)?;
            let target_node = graph.get_node(target_id)?;

            let source_center = source_node.center()?;
            let target_center = target_node.center()?;

            let start = calculate_exit_point(source_node, &target_center, direction);
            let end = calculate_entry_point(target_node, &source_center, direction);

            let bend_points = calculate_bend_points(&start, &end, direction);
            let label_position = if edge.label.is_some() {
                Some(calculate_label_position(&bend_points))
            } else {
                None
            };

            Some(EdgeRoutingInfo {
                edge_idx: idx,
                bend_points,
                label_position,
            })
        })
        .collect();

    // Second pass: apply routing to edges (mutable borrow)
    for info in routing_info {
        if let Some(edge) = graph.edges.get_mut(info.edge_idx) {
            edge.bend_points = info.bend_points;
            edge.label_position = info.label_position;
        }
    }
}

/// Calculate the exit point from a node toward a target
fn calculate_exit_point(
    node: &crate::layout::LayoutNode,
    _target: &Point,
    direction: LayoutDirection,
) -> Point {
    let (x, y) = (node.x.unwrap_or(0.0), node.y.unwrap_or(0.0));
    let (w, h) = (node.width, node.height);
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;

    match direction {
        LayoutDirection::TopToBottom => {
            // Exit from bottom center
            Point::new(cx, y + h)
        }
        LayoutDirection::BottomToTop => {
            // Exit from top center
            Point::new(cx, y)
        }
        LayoutDirection::LeftToRight => {
            // Exit from right center
            Point::new(x + w, cy)
        }
        LayoutDirection::RightToLeft => {
            // Exit from left center
            Point::new(x, cy)
        }
    }
}

/// Calculate the entry point into a node from a source
fn calculate_entry_point(
    node: &crate::layout::LayoutNode,
    _source: &Point,
    direction: LayoutDirection,
) -> Point {
    let (x, y) = (node.x.unwrap_or(0.0), node.y.unwrap_or(0.0));
    let (w, h) = (node.width, node.height);
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;

    match direction {
        LayoutDirection::TopToBottom => {
            // Enter from top center
            Point::new(cx, y)
        }
        LayoutDirection::BottomToTop => {
            // Enter from bottom center
            Point::new(cx, y + h)
        }
        LayoutDirection::LeftToRight => {
            // Enter from left center
            Point::new(x, cy)
        }
        LayoutDirection::RightToLeft => {
            // Enter from right center
            Point::new(x + w, cy)
        }
    }
}

/// Calculate bend points for orthogonal edge routing
fn calculate_bend_points(start: &Point, end: &Point, direction: LayoutDirection) -> Vec<Point> {
    let mut points = vec![*start];

    // For simple cases, use direct connection or single bend
    let is_horizontal = direction.is_horizontal();

    if is_horizontal {
        // Horizontal layout: edges go left-right
        if (start.x - end.x).abs() > 1.0 {
            // Need a bend if not aligned
            if (start.y - end.y).abs() > 1.0 {
                // S-curve: go horizontal to midpoint, then vertical, then horizontal
                let mid_x = (start.x + end.x) / 2.0;
                points.push(Point::new(mid_x, start.y));
                points.push(Point::new(mid_x, end.y));
            }
        }
    } else {
        // Vertical layout: edges go top-bottom
        if (start.y - end.y).abs() > 1.0 {
            // Need a bend if not aligned
            if (start.x - end.x).abs() > 1.0 {
                // S-curve: go vertical to midpoint, then horizontal, then vertical
                let mid_y = (start.y + end.y) / 2.0;
                points.push(Point::new(start.x, mid_y));
                points.push(Point::new(end.x, mid_y));
            }
        }
    }

    points.push(*end);
    points
}

/// Calculate label position (midpoint of edge path)
fn calculate_label_position(points: &[Point]) -> Point {
    if points.is_empty() {
        return Point::default();
    }

    if points.len() == 1 {
        return points[0];
    }

    // Find the middle segment
    let mid_idx = points.len() / 2;

    if points.len() % 2 == 0 {
        // Even number of points - midpoint between two middle points
        let p1 = &points[mid_idx - 1];
        let p2 = &points[mid_idx];
        Point::new((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0)
    } else {
        // Odd number of points - use the middle point
        points[mid_idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{LayoutEdge, LayoutNode, LayoutOptions};

    #[test]
    fn test_simple_vertical_routing() {
        let mut graph = LayoutGraph::new("test");
        graph.options = LayoutOptions {
            direction: LayoutDirection::TopToBottom,
            ..Default::default()
        };

        let mut a = LayoutNode::new("A", 50.0, 30.0);
        a.x = Some(0.0);
        a.y = Some(0.0);
        graph.add_node(a);

        let mut b = LayoutNode::new("B", 50.0, 30.0);
        b.x = Some(0.0);
        b.y = Some(100.0);
        graph.add_node(b);

        graph.add_edge(LayoutEdge::new("e1", "A", "B"));

        route_edges(&mut graph);

        let edge = &graph.edges[0];
        assert!(!edge.bend_points.is_empty());

        // Start should be at bottom of A
        let start = &edge.bend_points[0];
        assert!((start.y - 30.0).abs() < 1.0); // Bottom of A

        // End should be at top of B
        let end = edge.bend_points.last().unwrap();
        assert!((end.y - 100.0).abs() < 1.0); // Top of B
    }

    #[test]
    fn test_horizontal_routing() {
        let mut graph = LayoutGraph::new("test");
        graph.options = LayoutOptions {
            direction: LayoutDirection::LeftToRight,
            ..Default::default()
        };

        let mut a = LayoutNode::new("A", 50.0, 30.0);
        a.x = Some(0.0);
        a.y = Some(0.0);
        graph.add_node(a);

        let mut b = LayoutNode::new("B", 50.0, 30.0);
        b.x = Some(100.0);
        b.y = Some(0.0);
        graph.add_node(b);

        graph.add_edge(LayoutEdge::new("e1", "A", "B"));

        route_edges(&mut graph);

        let edge = &graph.edges[0];
        assert!(!edge.bend_points.is_empty());

        // Start should be at right of A
        let start = &edge.bend_points[0];
        assert!((start.x - 50.0).abs() < 1.0); // Right of A

        // End should be at left of B
        let end = edge.bend_points.last().unwrap();
        assert!((end.x - 100.0).abs() < 1.0); // Left of B
    }

    #[test]
    fn test_label_positioning() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(50.0, 0.0),
            Point::new(50.0, 50.0),
            Point::new(100.0, 50.0),
        ];

        let label_pos = calculate_label_position(&points);

        // Should be between point 1 and point 2
        assert!((label_pos.x - 50.0).abs() < 1.0);
        assert!((label_pos.y - 25.0).abs() < 1.0);
    }
}
