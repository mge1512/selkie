//! State diagram renderer

use std::collections::HashMap;

use crate::diagrams::state::{Direction, NotePosition, State, StateDb, StateType};
use crate::error::Result;
use crate::render::svg::{Attrs, RenderConfig, SvgDocument, SvgElement};

/// Render a state diagram to SVG
pub fn render_state(db: &StateDb, config: &RenderConfig) -> Result<String> {
    let mut doc = SvgDocument::new();

    // Layout constants
    let state_width = 120.0;
    let state_height = 40.0;
    let state_spacing_x = 80.0;
    let state_spacing_y = 60.0;
    let margin = 50.0;
    let start_end_radius = 12.0;
    let fork_join_width = 8.0;
    let fork_join_height = 60.0;

    // Determine which [*] states are starts vs ends based on transitions
    let start_end_states = determine_start_end_states(db);

    let states = db.get_states();

    if states.is_empty() {
        // Empty diagram
        doc.set_size(400.0, 200.0);
        if !db.diagram_title.is_empty() {
            let title_elem = SvgElement::Text {
                x: 200.0,
                y: 30.0,
                content: db.diagram_title.clone(),
                attrs: Attrs::new()
                    .with_attr("text-anchor", "middle")
                    .with_class("state-title")
                    .with_attr("font-size", "20")
                    .with_attr("font-weight", "bold"),
            };
            doc.add_element(title_elem);
        }
        return Ok(doc.to_string());
    }

    // Calculate positions for states using simple grid layout
    let mut state_positions: HashMap<String, (f64, f64)> = HashMap::new();
    let direction = db.get_direction();
    let is_horizontal = direction == Direction::LeftToRight || direction == Direction::RightToLeft;

    // Sort states to get consistent ordering
    let mut sorted_states: Vec<_> = states.iter().collect();
    sorted_states.sort_by(|a, b| a.0.cmp(b.0));

    let cols_per_row = if is_horizontal {
        sorted_states.len()
    } else {
        ((sorted_states.len() as f64).sqrt().ceil() as usize).max(1)
    };

    let mut max_width = margin;
    let mut max_height = margin;

    // Title offset
    let title_offset = if !db.diagram_title.is_empty() { 40.0 } else { 0.0 };

    for (i, (id, state)) in sorted_states.iter().enumerate() {
        let row = i / cols_per_row;
        let col = i % cols_per_row;

        let x = margin + (col as f64) * (state_width + state_spacing_x);
        let y = margin + title_offset + (row as f64) * (state_height + state_spacing_y);

        state_positions.insert((*id).clone(), (x, y));

        max_width = max_width.max(x + state_width + margin);
        max_height = max_height.max(y + state_height + margin);
    }

    doc.set_size(max_width, max_height);

    // Add theme styles
    if config.embed_css {
        doc.add_style(&config.theme.generate_css());
        doc.add_style(&generate_state_css());
    }

    // Add arrow marker
    doc.add_defs(vec![create_arrow_marker()]);

    // Render title
    if !db.diagram_title.is_empty() {
        let title_elem = SvgElement::Text {
            x: max_width / 2.0,
            y: 25.0,
            content: db.diagram_title.clone(),
            attrs: Attrs::new()
                .with_attr("text-anchor", "middle")
                .with_class("state-title")
                .with_attr("font-size", "20")
                .with_attr("font-weight", "bold"),
        };
        doc.add_element(title_elem);
    }

    // Render each state
    for (id, state) in &sorted_states {
        if let Some(&(x, y)) = state_positions.get(*id) {
            // Check if this [*] state is a start or end
            let is_end_state = start_end_states.get(*id).copied().unwrap_or(false);

            let state_elem = render_state_node(
                state,
                x,
                y,
                state_width,
                state_height,
                start_end_radius,
                fork_join_width,
                fork_join_height,
                is_end_state,
            );
            doc.add_element(state_elem);

            // Render note if present
            if let Some(note) = &state.note {
                let note_x = match note.position {
                    NotePosition::LeftOf => x - 120.0,
                    NotePosition::RightOf => x + state_width + 20.0,
                };
                let note_elem = render_note(note_x, y, &note.text);
                doc.add_element(note_elem);
            }
        }
    }

    // Render transitions
    for relation in db.get_relations() {
        if let (Some(&(x1, y1)), Some(&(x2, y2))) = (
            state_positions.get(&relation.state1),
            state_positions.get(&relation.state2),
        ) {
            let state1 = states.get(&relation.state1);
            let state2 = states.get(&relation.state2);

            let transition_elem = render_transition(
                x1,
                y1,
                x2,
                y2,
                state_width,
                state_height,
                start_end_radius,
                fork_join_width,
                fork_join_height,
                state1.map(|s| s.state_type),
                state2.map(|s| s.state_type),
                relation.description.as_deref(),
            );
            doc.add_element(transition_elem);
        }
    }

    Ok(doc.to_string())
}

/// Render a state node based on its type
fn render_state_node(
    state: &State,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    start_end_radius: f64,
    fork_join_width: f64,
    fork_join_height: f64,
    is_end_state: bool,
) -> SvgElement {
    let mut children = Vec::new();

    match state.state_type {
        StateType::Start => {
            // Filled circle for start state
            children.push(SvgElement::Circle {
                cx: x + width / 2.0,
                cy: y + height / 2.0,
                r: start_end_radius,
                attrs: Attrs::new()
                    .with_fill("#333333")
                    .with_class("state-start"),
            });
        }
        StateType::End => {
            render_end_state_bullseye(&mut children, x, y, width, height, start_end_radius);
        }
        StateType::Fork | StateType::Join => {
            // Black bar for fork/join
            children.push(SvgElement::Rect {
                x: x + (width - fork_join_width) / 2.0,
                y: y + (height - fork_join_height) / 2.0,
                width: fork_join_width,
                height: fork_join_height.min(height),
                rx: Some(2.0),
                ry: Some(2.0),
                attrs: Attrs::new()
                    .with_fill("#333333")
                    .with_class("state-fork-join"),
            });
        }
        StateType::Choice => {
            // Diamond for choice/decision
            let cx = x + width / 2.0;
            let cy = y + height / 2.0;
            let size = height / 2.0;

            children.push(SvgElement::Polygon {
                points: vec![
                    crate::layout::Point { x: cx, y: cy - size },
                    crate::layout::Point { x: cx + size, y: cy },
                    crate::layout::Point { x: cx, y: cy + size },
                    crate::layout::Point { x: cx - size, y: cy },
                ],
                attrs: Attrs::new()
                    .with_fill("#ECECFF")
                    .with_stroke("#333333")
                    .with_stroke_width(1.0)
                    .with_class("state-choice"),
            });
        }
        StateType::Divider => {
            // Horizontal line for divider
            children.push(SvgElement::Line {
                x1: x,
                y1: y + height / 2.0,
                x2: x + width,
                y2: y + height / 2.0,
                attrs: Attrs::new()
                    .with_stroke("#333333")
                    .with_stroke_width(2.0)
                    .with_stroke_dasharray("5,5")
                    .with_class("state-divider"),
            });
        }
        StateType::Default => {
            // Check if this is [*] which could be start or end
            if state.id == "[*]" {
                if is_end_state {
                    // End state: double circle (bullseye)
                    render_end_state_bullseye(&mut children, x, y, width, height, start_end_radius);
                } else {
                    // Start state: filled circle
                    children.push(SvgElement::Circle {
                        cx: x + width / 2.0,
                        cy: y + height / 2.0,
                        r: start_end_radius,
                        attrs: Attrs::new()
                            .with_fill("#333333")
                            .with_class("state-start"),
                    });
                }
            } else {
                // Rounded rectangle for regular state
                children.push(SvgElement::Rect {
                    x,
                    y,
                    width,
                    height,
                    rx: Some(10.0),
                    ry: Some(10.0),
                    attrs: Attrs::new()
                        .with_fill("#ECECFF")
                        .with_stroke("#333333")
                        .with_stroke_width(1.0)
                        .with_class("state-box"),
                });

                // State label
                let label = state.alias.as_ref().unwrap_or(&state.id);
                children.push(SvgElement::Text {
                    x: x + width / 2.0,
                    y: y + height / 2.0 + 5.0,
                    content: label.clone(),
                    attrs: Attrs::new()
                        .with_attr("text-anchor", "middle")
                        .with_class("state-label")
                        .with_attr("font-size", "12"),
                });

                // State descriptions
                if !state.descriptions.is_empty() {
                    let desc_y = y + height / 2.0 + 18.0;
                    for (i, desc) in state.descriptions.iter().enumerate() {
                        children.push(SvgElement::Text {
                            x: x + width / 2.0,
                            y: desc_y + (i as f64) * 14.0,
                            content: desc.clone(),
                            attrs: Attrs::new()
                                .with_attr("text-anchor", "middle")
                                .with_class("state-description")
                                .with_attr("font-size", "10"),
                        });
                    }
                }
            }
        }
    }

    SvgElement::Group {
        children,
        attrs: Attrs::new()
            .with_class("state-node")
            .with_id(&format!("state-{}", state.id)),
    }
}

/// Render a transition between two states
fn render_transition(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    state_width: f64,
    state_height: f64,
    start_end_radius: f64,
    _fork_join_width: f64,
    _fork_join_height: f64,
    state1_type: Option<StateType>,
    state2_type: Option<StateType>,
    label: Option<&str>,
) -> SvgElement {
    let mut children = Vec::new();

    // Calculate connection points based on state types
    let (start_x, start_y) = calculate_exit_point(
        x1,
        y1,
        state_width,
        state_height,
        x2 + state_width / 2.0,
        y2 + state_height / 2.0,
        state1_type,
        start_end_radius,
    );

    let (end_x, end_y) = calculate_entry_point(
        x2,
        y2,
        state_width,
        state_height,
        x1 + state_width / 2.0,
        y1 + state_height / 2.0,
        state2_type,
        start_end_radius,
    );

    // Transition line
    children.push(SvgElement::Line {
        x1: start_x,
        y1: start_y,
        x2: end_x,
        y2: end_y,
        attrs: Attrs::new()
            .with_stroke("#333333")
            .with_stroke_width(1.0)
            .with_attr("marker-end", "url(#arrow)")
            .with_class("transition-line"),
    });

    // Transition label
    if let Some(text) = label {
        if !text.is_empty() {
            let mid_x = (start_x + end_x) / 2.0;
            let mid_y = (start_y + end_y) / 2.0;

            children.push(SvgElement::Text {
                x: mid_x,
                y: mid_y - 5.0,
                content: text.to_string(),
                attrs: Attrs::new()
                    .with_attr("text-anchor", "middle")
                    .with_class("transition-label")
                    .with_attr("font-size", "11"),
            });
        }
    }

    SvgElement::Group {
        children,
        attrs: Attrs::new().with_class("transition"),
    }
}

/// Calculate exit point from a state
fn calculate_exit_point(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    target_x: f64,
    target_y: f64,
    state_type: Option<StateType>,
    start_end_radius: f64,
) -> (f64, f64) {
    let cx = x + width / 2.0;
    let cy = y + height / 2.0;

    match state_type {
        Some(StateType::Start) | Some(StateType::End) => {
            // Circle - calculate intersection
            let dx = target_x - cx;
            let dy = target_y - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > 0.0 {
                (cx + dx / dist * start_end_radius, cy + dy / dist * start_end_radius)
            } else {
                (cx + start_end_radius, cy)
            }
        }
        _ => {
            // Rectangle - calculate edge intersection
            let dx = target_x - cx;
            let dy = target_y - cy;

            if dx.abs() > dy.abs() {
                if dx > 0.0 {
                    (x + width, cy)
                } else {
                    (x, cy)
                }
            } else if dy > 0.0 {
                (cx, y + height)
            } else {
                (cx, y)
            }
        }
    }
}

/// Calculate entry point into a state
fn calculate_entry_point(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    source_x: f64,
    source_y: f64,
    state_type: Option<StateType>,
    start_end_radius: f64,
) -> (f64, f64) {
    let cx = x + width / 2.0;
    let cy = y + height / 2.0;

    match state_type {
        Some(StateType::Start) | Some(StateType::End) => {
            // Circle - calculate intersection
            let dx = source_x - cx;
            let dy = source_y - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > 0.0 {
                (cx + dx / dist * start_end_radius, cy + dy / dist * start_end_radius)
            } else {
                (cx - start_end_radius, cy)
            }
        }
        _ => {
            // Rectangle - calculate edge intersection
            let dx = source_x - cx;
            let dy = source_y - cy;

            if dx.abs() > dy.abs() {
                if dx > 0.0 {
                    (x + width, cy)
                } else {
                    (x, cy)
                }
            } else if dy > 0.0 {
                (cx, y + height)
            } else {
                (cx, y)
            }
        }
    }
}

/// Render a note
fn render_note(x: f64, y: f64, text: &str) -> SvgElement {
    let note_width = 100.0;
    let note_height = 40.0;
    let fold_size = 8.0;

    let mut children = Vec::new();

    // Note box with folded corner
    let path = format!(
        "M {} {} L {} {} L {} {} L {} {} L {} {} Z",
        x,
        y,
        x + note_width - fold_size,
        y,
        x + note_width,
        y + fold_size,
        x + note_width,
        y + note_height,
        x,
        y + note_height
    );

    children.push(SvgElement::Path {
        d: path,
        attrs: Attrs::new()
            .with_fill("#FFFFCC")
            .with_stroke("#333333")
            .with_stroke_width(1.0)
            .with_class("note-box"),
    });

    // Fold line
    let fold_path = format!(
        "M {} {} L {} {} L {} {}",
        x + note_width - fold_size,
        y,
        x + note_width - fold_size,
        y + fold_size,
        x + note_width,
        y + fold_size
    );

    children.push(SvgElement::Path {
        d: fold_path,
        attrs: Attrs::new()
            .with_fill("none")
            .with_stroke("#333333")
            .with_stroke_width(1.0),
    });

    // Note text
    children.push(SvgElement::Text {
        x: x + note_width / 2.0,
        y: y + note_height / 2.0 + 4.0,
        content: text.to_string(),
        attrs: Attrs::new()
            .with_attr("text-anchor", "middle")
            .with_class("note-text")
            .with_attr("font-size", "11"),
    });

    SvgElement::Group {
        children,
        attrs: Attrs::new().with_class("note"),
    }
}

/// Create arrow marker
fn create_arrow_marker() -> SvgElement {
    SvgElement::Marker {
        id: "arrow".to_string(),
        view_box: "0 0 10 10".to_string(),
        ref_x: 10.0,
        ref_y: 5.0,
        marker_width: 6.0,
        marker_height: 6.0,
        orient: "auto".to_string(),
        marker_units: None,
        children: vec![SvgElement::Path {
            d: "M 0 0 L 10 5 L 0 10 z".to_string(),
            attrs: Attrs::new()
                .with_fill("#333333")
                .with_stroke("#333333")
                .with_stroke_width(1.0),
        }],
    }
}

/// Determine which [*] states are end states based on transitions
/// A [*] state is an end state if it's the target of a transition but not the source
fn determine_start_end_states(db: &StateDb) -> HashMap<String, bool> {
    let mut result = HashMap::new();
    let relations = db.get_relations();

    // Track which [*] states are sources vs targets
    let mut is_source = HashMap::new();
    let mut is_target = HashMap::new();

    for relation in relations {
        if relation.state1 == "[*]" {
            is_source.insert("[*]".to_string(), true);
        }
        if relation.state2 == "[*]" {
            is_target.insert("[*]".to_string(), true);
        }
    }

    // A [*] that is only a target (not a source) is an end state
    // A [*] that is only a source (not a target) is a start state
    // A [*] that is both... could be either. For now, if it has incoming transitions, treat as end.
    if is_target.contains_key("[*]") && !is_source.contains_key("[*]") {
        result.insert("[*]".to_string(), true); // is_end_state = true
    } else if is_target.contains_key("[*]") {
        // Has both incoming and outgoing - treat as end state since it has incoming
        result.insert("[*]".to_string(), true);
    }

    result
}

/// Render end state bullseye (double circle)
fn render_end_state_bullseye(
    children: &mut Vec<SvgElement>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    start_end_radius: f64,
) {
    // Outer circle
    children.push(SvgElement::Circle {
        cx: x + width / 2.0,
        cy: y + height / 2.0,
        r: start_end_radius,
        attrs: Attrs::new()
            .with_fill("none")
            .with_stroke("#333333")
            .with_stroke_width(2.0)
            .with_class("state-end-outer"),
    });
    // Inner circle
    children.push(SvgElement::Circle {
        cx: x + width / 2.0,
        cy: y + height / 2.0,
        r: start_end_radius * 0.6,
        attrs: Attrs::new()
            .with_fill("#333333")
            .with_class("state-end-inner"),
    });
}

fn generate_state_css() -> String {
    r#"
.state-title {
  fill: #333333;
}

.state-box {
  fill: #ECECFF;
  stroke: #333333;
}

.state-label {
  fill: #333333;
}

.state-description {
  fill: #666666;
}

.state-start {
  fill: #333333;
}

.state-end-outer {
  stroke: #333333;
}

.state-end-inner {
  fill: #333333;
}

.state-fork-join {
  fill: #333333;
}

.state-choice {
  fill: #ECECFF;
  stroke: #333333;
}

.state-divider {
  stroke: #333333;
  stroke-dasharray: 5, 5;
}

.transition-line {
  stroke: #333333;
}

.transition-label {
  fill: #333333;
}

.note-box {
  fill: #FFFFCC;
  stroke: #333333;
}

.note-text {
  fill: #333333;
}
"#
    .to_string()
}
