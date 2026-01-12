//! Edge rendering for flowcharts

use crate::diagrams::flowchart::{EdgeStroke, FlowEdge};
use crate::layout::LayoutEdge;

use super::elements::{Attrs, SvgElement};
use super::markers;
use super::theme::Theme;

/// Render an edge
pub fn render_edge(layout_edge: &LayoutEdge, flow_edge: &FlowEdge, _theme: &Theme) -> SvgElement {
    let mut elements = Vec::new();

    // Build path from bend points - use curved path for smooth corners
    if !layout_edge.bend_points.is_empty() {
        let path_d = build_curved_path(&layout_edge.bend_points);

        let mut attrs = Attrs::new()
            .with_class("edge-path")
            .with_fill("none");

        // Apply stroke style
        match flow_edge.stroke {
            EdgeStroke::Normal => {
                attrs = attrs.with_stroke_width(2.0);
            }
            EdgeStroke::Thick => {
                attrs = attrs.with_stroke_width(3.5);
            }
            EdgeStroke::Dotted => {
                attrs = attrs
                    .with_stroke_width(2.0)
                    .with_stroke_dasharray("3,3");
            }
            EdgeStroke::Invisible => {
                attrs = attrs.with_stroke_width(0.0);
            }
        }

        // Apply arrow markers
        if let Some(marker_url) = markers::get_marker_url(flow_edge.edge_type.as_deref()) {
            attrs = attrs.with_attr("marker-end", &marker_url);
        }
        if let Some(start_marker_url) = markers::get_start_marker_url(flow_edge.edge_type.as_deref()) {
            attrs = attrs.with_attr("marker-start", &start_marker_url);
        }

        elements.push(SvgElement::path(path_d).with_attrs(attrs));
    }

    // Add label if present
    if !flow_edge.text.is_empty() {
        if let Some(label_pos) = &layout_edge.label_position {
            // Estimate text width (approximately 8px per character for typical fonts)
            let text_width = flow_edge.text.len() as f64 * 8.0;
            let text_height = 16.0; // Typical line height
            let padding = 4.0; // Padding around the text

            // Add background rect first (translucent gray like mermaid.js)
            let bg_attrs = Attrs::new()
                .with_class("edge-label-bg")
                .with_fill("#e8e8e8")
                .with_attr("fill-opacity", "0.8");

            elements.push(
                SvgElement::rect(
                    label_pos.x - text_width / 2.0 - padding,
                    label_pos.y - text_height / 2.0 - padding / 2.0,
                    text_width + padding * 2.0,
                    text_height + padding,
                )
                .with_attrs(bg_attrs),
            );

            // Then add the text on top
            let label_attrs = Attrs::new()
                .with_class("edge-label")
                .with_attr("text-anchor", "middle")
                .with_attr("dominant-baseline", "central");

            elements.push(
                SvgElement::text(label_pos.x, label_pos.y, &flow_edge.text)
                    .with_attrs(label_attrs),
            );
        }
    }

    let group_attrs = Attrs::new()
        .with_class("edge")
        .with_id(&format!("edge-{}", layout_edge.id));

    SvgElement::group(elements).with_attrs(group_attrs)
}

/// Build SVG path from bend points (straight lines)
fn build_path(points: &[crate::layout::Point]) -> String {
    if points.is_empty() {
        return String::new();
    }

    let mut d = String::new();

    // Move to first point
    d.push_str(&format!("M {} {}", points[0].x, points[0].y));

    // Line to each subsequent point
    for point in &points[1..] {
        d.push_str(&format!(" L {} {}", point.x, point.y));
    }

    d
}

/// Build curved SVG path from bend points using quadratic bezier curves
/// This creates smooth curves at corners like mermaid.js's curveBasis
fn build_curved_path(points: &[crate::layout::Point]) -> String {
    if points.is_empty() {
        return String::new();
    }

    if points.len() < 3 {
        // With fewer than 3 points, just use a straight line
        return build_path(points);
    }

    let mut d = String::new();

    // Move to first point
    d.push_str(&format!("M {} {}", points[0].x, points[0].y));

    // For each corner point, use a quadratic bezier curve
    // The corner becomes the control point, and we curve through it
    for i in 1..points.len() - 1 {
        let prev = &points[i - 1];
        let curr = &points[i];
        let next = &points[i + 1];

        // Line to a point before the corner
        let t = 0.5; // How far to extend before curving
        let pre_corner_x = prev.x + (curr.x - prev.x) * t;
        let pre_corner_y = prev.y + (curr.y - prev.y) * t;

        // Use quadratic bezier: corner as control point, midway to next as end
        let post_corner_x = curr.x + (next.x - curr.x) * (1.0 - t);
        let post_corner_y = curr.y + (next.y - curr.y) * (1.0 - t);

        if i == 1 {
            // First segment: line from start to pre-corner
            d.push_str(&format!(" L {} {}", pre_corner_x, pre_corner_y));
        }

        // Quadratic bezier curve around the corner
        d.push_str(&format!(
            " Q {} {} {} {}",
            curr.x, curr.y, post_corner_x, post_corner_y
        ));

        // Note: next corner transitions are handled in the next iteration
    }

    // Line to the last point
    let last = points.last().unwrap();
    d.push_str(&format!(" L {} {}", last.x, last.y));

    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::Point;

    #[test]
    fn test_build_path() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(50.0, 0.0),
            Point::new(50.0, 50.0),
        ];

        let path = build_path(&points);
        assert_eq!(path, "M 0 0 L 50 0 L 50 50");
    }

    #[test]
    fn test_empty_path() {
        let points: Vec<Point> = vec![];
        let path = build_path(&points);
        assert!(path.is_empty());
    }

    #[test]
    fn test_build_curved_path_contains_bezier() {
        // Curved paths should use quadratic bezier (Q) or cubic bezier (C) commands
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(50.0, 0.0),
            Point::new(50.0, 50.0),
            Point::new(100.0, 50.0),
        ];

        let path = build_curved_path(&points);

        // Should start with M (move to)
        assert!(path.starts_with("M"), "Path should start with M command");
        // Should contain curve commands (Q for quadratic bezier or C for cubic)
        assert!(
            path.contains("Q") || path.contains("C") || path.contains("S"),
            "Curved path should contain bezier curve commands, got: {}",
            path
        );
        // Should NOT be all straight lines
        let l_count = path.matches(" L ").count();
        assert!(l_count < points.len() - 1, "Curved path should not use only L commands");
    }

    #[test]
    fn test_build_curved_path_two_points() {
        // With only two points, should be a straight line (no curve possible)
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 100.0),
        ];

        let path = build_curved_path(&points);
        assert!(path.starts_with("M"));
        assert!(path.contains("L") || path.contains("100"));
    }

    #[test]
    fn test_edge_label_has_background_rect() {
        use crate::diagrams::flowchart::{FlowEdge, EdgeStroke, FlowTextType};
        use std::collections::HashMap;

        let layout_edge = LayoutEdge {
            id: "e1".to_string(),
            sources: vec!["a".to_string()],
            targets: vec!["b".to_string()],
            label: Some("label".to_string()),
            bend_points: vec![
                Point::new(0.0, 0.0),
                Point::new(100.0, 100.0),
            ],
            label_position: Some(Point::new(50.0, 50.0)),
            weight: 1,
            reversed: false,
            metadata: HashMap::new(),
        };

        let flow_edge = FlowEdge {
            id: None,
            is_user_defined_id: false,
            start: "a".to_string(),
            end: "b".to_string(),
            interpolate: None,
            edge_type: Some("arrow_point".to_string()),
            stroke: EdgeStroke::Normal,
            style: vec![],
            length: None,
            text: "label".to_string(),
            label_type: FlowTextType::Text,
            classes: vec![],
            animation: None,
            animate: None,
        };

        let theme = Theme::default();
        let edge_element = render_edge(&layout_edge, &flow_edge, &theme);
        let svg = edge_element.to_svg(0);

        // Edge label should have a background rect before the text
        assert!(
            svg.contains("<rect") && svg.contains("<text"),
            "Edge label should have background rect, got: {}",
            svg
        );

        // The rect should have some opacity for the translucent background
        assert!(
            svg.contains("opacity") || svg.contains("fill-opacity"),
            "Edge label background should have opacity for translucent effect"
        );
    }
}
