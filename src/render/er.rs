//! Entity Relationship diagram renderer

use std::collections::HashMap;

use crate::diagrams::er::{Cardinality, Direction, ErDb, Entity, Identification};
use crate::error::Result;
use crate::render::svg::{Attrs, RenderConfig, SvgDocument, SvgElement};

/// Render an ER diagram to SVG
pub fn render_er(db: &ErDb, config: &RenderConfig) -> Result<String> {
    let mut doc = SvgDocument::new();

    // Layout constants
    let entity_width = 160.0;
    let entity_header_height = 30.0;
    let attr_row_height = 20.0;
    let entity_spacing_x = 100.0;
    let entity_spacing_y = 80.0;
    let margin = 50.0;
    let padding = 8.0;

    let entities = db.get_entities();

    if entities.is_empty() {
        // Empty diagram
        doc.set_size(400.0, 200.0);
        if !db.diagram_title.is_empty() {
            let title_elem = SvgElement::Text {
                x: 200.0,
                y: 30.0,
                content: db.diagram_title.clone(),
                attrs: Attrs::new()
                    .with_attr("text-anchor", "middle")
                    .with_class("er-title")
                    .with_attr("font-size", "20")
                    .with_attr("font-weight", "bold"),
            };
            doc.add_element(title_elem);
        }
        return Ok(doc.to_string());
    }

    // Calculate entity heights
    let mut entity_heights: HashMap<String, f64> = HashMap::new();
    for (name, entity) in entities {
        let height = entity_header_height + (entity.attributes.len() as f64) * attr_row_height + padding * 2.0;
        entity_heights.insert(name.clone(), height.max(entity_header_height + padding * 2.0));
    }

    // Sort entities for consistent ordering
    let mut sorted_entities: Vec<_> = entities.iter().collect();
    sorted_entities.sort_by(|a, b| a.0.cmp(b.0));

    // Calculate positions using grid layout
    let direction = db.get_direction();
    let is_horizontal = direction == Direction::LeftToRight || direction == Direction::RightToLeft;

    let cols_per_row = if is_horizontal {
        sorted_entities.len()
    } else {
        ((sorted_entities.len() as f64).sqrt().ceil() as usize).max(1)
    };

    let mut entity_positions: HashMap<String, (f64, f64)> = HashMap::new();
    let mut max_width = margin;
    let mut max_height = margin;

    // Title offset
    let title_offset = if !db.diagram_title.is_empty() { 40.0 } else { 0.0 };

    for (i, (name, _entity)) in sorted_entities.iter().enumerate() {
        let row = i / cols_per_row;
        let col = i % cols_per_row;

        // Calculate row height
        let row_start = row * cols_per_row;
        let row_end = ((row + 1) * cols_per_row).min(sorted_entities.len());
        let row_height: f64 = (row_start..row_end)
            .map(|j| {
                entity_heights
                    .get(sorted_entities[j].0)
                    .copied()
                    .unwrap_or(entity_header_height)
            })
            .fold(0.0, f64::max);

        let x = margin + (col as f64) * (entity_width + entity_spacing_x);
        let y = margin + title_offset + (row as f64) * (row_height + entity_spacing_y);

        entity_positions.insert((*name).clone(), (x, y));

        let height = entity_heights.get(*name).copied().unwrap_or(entity_header_height);
        max_width = max_width.max(x + entity_width + margin);
        max_height = max_height.max(y + height + margin);
    }

    doc.set_size(max_width, max_height);

    // Add theme styles
    if config.embed_css {
        doc.add_style(&config.theme.generate_css());
        doc.add_style(&generate_er_css());
    }

    // Render title
    if !db.diagram_title.is_empty() {
        let title_elem = SvgElement::Text {
            x: max_width / 2.0,
            y: 25.0,
            content: db.diagram_title.clone(),
            attrs: Attrs::new()
                .with_attr("text-anchor", "middle")
                .with_class("er-title")
                .with_attr("font-size", "20")
                .with_attr("font-weight", "bold"),
        };
        doc.add_element(title_elem);
    }

    // Render each entity
    for (name, entity) in &sorted_entities {
        if let Some(&(x, y)) = entity_positions.get(*name) {
            let height = entity_heights.get(*name).copied().unwrap_or(entity_header_height);
            let entity_elem = render_entity(
                entity,
                x,
                y,
                entity_width,
                height,
                entity_header_height,
                attr_row_height,
                padding,
            );
            doc.add_element(entity_elem);
        }
    }

    // Create entity id to name mapping for relationship rendering
    let entity_id_to_name: HashMap<String, String> = entities
        .iter()
        .map(|(name, entity)| (entity.id.clone(), name.clone()))
        .collect();

    // Render relationships
    for relationship in db.get_relationships() {
        // Look up entity names from IDs
        let entity_a_name = entity_id_to_name.get(&relationship.entity_a);
        let entity_b_name = entity_id_to_name.get(&relationship.entity_b);

        if let (Some(a_name), Some(b_name)) = (entity_a_name, entity_b_name) {
            if let (Some(&(x1, y1)), Some(&(x2, y2))) = (
                entity_positions.get(a_name),
                entity_positions.get(b_name),
            ) {
                let h1 = entity_heights.get(a_name).copied().unwrap_or(entity_header_height);
                let h2 = entity_heights.get(b_name).copied().unwrap_or(entity_header_height);

                let rel_elem = render_relationship(
                    x1,
                    y1,
                    h1,
                    x2,
                    y2,
                    h2,
                    entity_width,
                    &relationship.role_a,
                    relationship.rel_spec.card_a,
                    relationship.rel_spec.card_b,
                    relationship.rel_spec.rel_type,
                );
                doc.add_element(rel_elem);
            }
        }
    }

    Ok(doc.to_string())
}

/// Render an entity box with attributes
fn render_entity(
    entity: &Entity,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    header_height: f64,
    attr_row_height: f64,
    padding: f64,
) -> SvgElement {
    let mut children = Vec::new();

    // Entity box
    children.push(SvgElement::Rect {
        x,
        y,
        width,
        height,
        rx: Some(0.0),
        ry: Some(0.0),
        attrs: Attrs::new()
            .with_fill("#ECECFF")
            .with_stroke("#333333")
            .with_stroke_width(1.0)
            .with_class("entity-box"),
    });

    // Header background
    children.push(SvgElement::Rect {
        x,
        y,
        width,
        height: header_height,
        rx: Some(0.0),
        ry: Some(0.0),
        attrs: Attrs::new()
            .with_fill("#9370DB")
            .with_stroke("#333333")
            .with_stroke_width(1.0)
            .with_class("entity-header"),
    });

    // Entity name
    let display_name = if !entity.alias.is_empty() {
        &entity.alias
    } else {
        &entity.label
    };
    children.push(SvgElement::Text {
        x: x + width / 2.0,
        y: y + header_height / 2.0 + 5.0,
        content: display_name.clone(),
        attrs: Attrs::new()
            .with_attr("text-anchor", "middle")
            .with_class("entity-name")
            .with_attr("font-size", "14")
            .with_attr("font-weight", "bold")
            .with_fill("#FFFFFF"),
    });

    // Attributes
    let mut attr_y = y + header_height + padding;
    for attr in &entity.attributes {
        attr_y += attr_row_height;

        // Key indicators (PK, FK, UK)
        let key_str = attr
            .keys
            .iter()
            .map(|k| k.as_str())
            .collect::<Vec<_>>()
            .join(",");

        let attr_text = if !key_str.is_empty() {
            format!("{} {} {}", attr.attr_type, attr.name, key_str)
        } else {
            format!("{} {}", attr.attr_type, attr.name)
        };

        children.push(SvgElement::Text {
            x: x + padding,
            y: attr_y - 4.0,
            content: attr_text,
            attrs: Attrs::new()
                .with_attr("text-anchor", "start")
                .with_class("entity-attr")
                .with_attr("font-size", "11"),
        });
    }

    SvgElement::Group {
        children,
        attrs: Attrs::new()
            .with_class("entity-node")
            .with_id(&entity.id),
    }
}

/// Render a relationship line between two entities
fn render_relationship(
    x1: f64,
    y1: f64,
    h1: f64,
    x2: f64,
    y2: f64,
    h2: f64,
    width: f64,
    label: &str,
    card_a: Cardinality,
    card_b: Cardinality,
    rel_type: Identification,
) -> SvgElement {
    let mut children = Vec::new();

    // Calculate connection points
    let (start_x, start_y, end_x, end_y) =
        calculate_connection_points(x1, y1, h1, x2, y2, h2, width);

    // Main relationship line
    let mut line_attrs = Attrs::new()
        .with_stroke("#333333")
        .with_stroke_width(1.0)
        .with_fill("none")
        .with_class("relationship-line");

    // Dotted line for non-identifying relationships
    if rel_type == Identification::NonIdentifying {
        line_attrs = line_attrs.with_stroke_dasharray("5,5");
    }

    children.push(SvgElement::Line {
        x1: start_x,
        y1: start_y,
        x2: end_x,
        y2: end_y,
        attrs: line_attrs,
    });

    // Calculate angle for cardinality symbols
    let dx = end_x - start_x;
    let dy = end_y - start_y;
    let angle = dy.atan2(dx);

    // Render cardinality symbols at start
    let start_card = render_cardinality(start_x, start_y, angle, card_a, false);
    children.push(start_card);

    // Render cardinality symbols at end
    let end_card = render_cardinality(end_x, end_y, angle, card_b, true);
    children.push(end_card);

    // Relationship label
    if !label.is_empty() {
        let mid_x = (start_x + end_x) / 2.0;
        let mid_y = (start_y + end_y) / 2.0;

        // Background for label
        let label_width = (label.len() as f64) * 7.0;
        children.push(SvgElement::Rect {
            x: mid_x - label_width / 2.0 - 4.0,
            y: mid_y - 10.0,
            width: label_width + 8.0,
            height: 16.0,
            rx: Some(2.0),
            ry: Some(2.0),
            attrs: Attrs::new()
                .with_fill("#FFFFFF")
                .with_stroke("#333333")
                .with_stroke_width(0.5),
        });

        children.push(SvgElement::Text {
            x: mid_x,
            y: mid_y + 3.0,
            content: label.to_string(),
            attrs: Attrs::new()
                .with_attr("text-anchor", "middle")
                .with_class("relationship-label")
                .with_attr("font-size", "10"),
        });
    }

    SvgElement::Group {
        children,
        attrs: Attrs::new().with_class("relationship"),
    }
}

/// Render cardinality symbol (crow's foot notation)
fn render_cardinality(x: f64, y: f64, angle: f64, card: Cardinality, at_end: bool) -> SvgElement {
    let mut children = Vec::new();
    let offset = if at_end { 0.0 } else { std::f64::consts::PI };
    let symbol_angle = angle + offset;

    let cos_a = symbol_angle.cos();
    let sin_a = symbol_angle.sin();

    // Distance from connection point
    let dist = 15.0;
    let foot_spread = 8.0;

    // Crow's foot end positions
    let base_x = x + dist * cos_a;
    let base_y = y + dist * sin_a;

    // Perpendicular direction for spread
    let perp_cos = (-sin_a) * foot_spread;
    let perp_sin = cos_a * foot_spread;

    match card {
        Cardinality::OnlyOne => {
            // Two vertical lines (||)
            let line_dist = 5.0;
            for i in [-1.0, 1.0] {
                let lx = base_x + i * line_dist * cos_a;
                let ly = base_y + i * line_dist * sin_a;
                children.push(SvgElement::Line {
                    x1: lx + perp_cos,
                    y1: ly + perp_sin,
                    x2: lx - perp_cos,
                    y2: ly - perp_sin,
                    attrs: Attrs::new()
                        .with_stroke("#333333")
                        .with_stroke_width(1.0),
                });
            }
        }
        Cardinality::ZeroOrOne => {
            // Circle and one line (o|)
            let circle_x = base_x + 10.0 * cos_a;
            let circle_y = base_y + 10.0 * sin_a;
            children.push(SvgElement::Circle {
                cx: circle_x,
                cy: circle_y,
                r: 5.0,
                attrs: Attrs::new()
                    .with_fill("none")
                    .with_stroke("#333333")
                    .with_stroke_width(1.0),
            });
            children.push(SvgElement::Line {
                x1: base_x + perp_cos,
                y1: base_y + perp_sin,
                x2: base_x - perp_cos,
                y2: base_y - perp_sin,
                attrs: Attrs::new()
                    .with_stroke("#333333")
                    .with_stroke_width(1.0),
            });
        }
        Cardinality::ZeroOrMore => {
            // Circle and crow's foot (o{)
            let circle_x = base_x + 15.0 * cos_a;
            let circle_y = base_y + 15.0 * sin_a;
            children.push(SvgElement::Circle {
                cx: circle_x,
                cy: circle_y,
                r: 5.0,
                attrs: Attrs::new()
                    .with_fill("none")
                    .with_stroke("#333333")
                    .with_stroke_width(1.0),
            });
            // Crow's foot
            let foot_x = base_x + 5.0 * cos_a;
            let foot_y = base_y + 5.0 * sin_a;
            children.push(SvgElement::Path {
                d: format!(
                    "M {} {} L {} {} M {} {} L {} {}",
                    x, y, foot_x + perp_cos, foot_y + perp_sin,
                    x, y, foot_x - perp_cos, foot_y - perp_sin
                ),
                attrs: Attrs::new()
                    .with_fill("none")
                    .with_stroke("#333333")
                    .with_stroke_width(1.0),
            });
        }
        Cardinality::OneOrMore => {
            // Line and crow's foot (|{)
            children.push(SvgElement::Line {
                x1: base_x + perp_cos,
                y1: base_y + perp_sin,
                x2: base_x - perp_cos,
                y2: base_y - perp_sin,
                attrs: Attrs::new()
                    .with_stroke("#333333")
                    .with_stroke_width(1.0),
            });
            // Crow's foot
            let foot_x = base_x + 5.0 * cos_a;
            let foot_y = base_y + 5.0 * sin_a;
            children.push(SvgElement::Path {
                d: format!(
                    "M {} {} L {} {} M {} {} L {} {}",
                    x, y, foot_x + perp_cos, foot_y + perp_sin,
                    x, y, foot_x - perp_cos, foot_y - perp_sin
                ),
                attrs: Attrs::new()
                    .with_fill("none")
                    .with_stroke("#333333")
                    .with_stroke_width(1.0),
            });
        }
        Cardinality::MdParent => {
            // Parent indicator
            children.push(SvgElement::Line {
                x1: base_x + perp_cos,
                y1: base_y + perp_sin,
                x2: base_x - perp_cos,
                y2: base_y - perp_sin,
                attrs: Attrs::new()
                    .with_stroke("#333333")
                    .with_stroke_width(2.0),
            });
        }
    }

    SvgElement::Group {
        children,
        attrs: Attrs::new().with_class("cardinality"),
    }
}

/// Calculate connection points on entity box edges
fn calculate_connection_points(
    x1: f64,
    y1: f64,
    h1: f64,
    x2: f64,
    y2: f64,
    h2: f64,
    width: f64,
) -> (f64, f64, f64, f64) {
    let center1_x = x1 + width / 2.0;
    let center1_y = y1 + h1 / 2.0;
    let center2_x = x2 + width / 2.0;
    let center2_y = y2 + h2 / 2.0;

    let dx = center2_x - center1_x;
    let dy = center2_y - center1_y;

    // Determine which edges to connect based on relative positions
    let (start_x, start_y) = if dx.abs() > dy.abs() {
        if dx > 0.0 {
            (x1 + width, center1_y)
        } else {
            (x1, center1_y)
        }
    } else if dy > 0.0 {
        (center1_x, y1 + h1)
    } else {
        (center1_x, y1)
    };

    let (end_x, end_y) = if dx.abs() > dy.abs() {
        if dx > 0.0 {
            (x2, center2_y)
        } else {
            (x2 + width, center2_y)
        }
    } else if dy > 0.0 {
        (center2_x, y2)
    } else {
        (center2_x, y2 + h2)
    };

    (start_x, start_y, end_x, end_y)
}

fn generate_er_css() -> String {
    r#"
.er-title {
  fill: #333333;
}

.entity-box {
  fill: #ECECFF;
  stroke: #333333;
}

.entity-header {
  fill: #9370DB;
  stroke: #333333;
}

.entity-name {
  fill: #FFFFFF;
  font-weight: bold;
}

.entity-attr {
  fill: #333333;
}

.relationship-line {
  stroke: #333333;
}

.relationship-label {
  fill: #333333;
}

.cardinality {
  stroke: #333333;
}
"#
    .to_string()
}
