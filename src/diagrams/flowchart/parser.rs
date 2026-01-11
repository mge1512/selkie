//! Flowchart parser
//!
//! This module provides parsing for flowchart diagrams using pest.

use pest::Parser;
use pest_derive::Parser;

use super::types::{FlowVertexType, FlowchartDb, FlowText};
use crate::error::{MermaidError, Result};

#[derive(Parser)]
#[grammar = "diagrams/flowchart/flowchart.pest"]
struct FlowchartParser;

/// Parse a flowchart diagram
pub fn parse(input: &str) -> Result<FlowchartDb> {
    let mut db = FlowchartDb::new();
    parse_into(input, &mut db)?;
    Ok(db)
}

/// Parse into an existing database
pub fn parse_into(input: &str, db: &mut FlowchartDb) -> Result<()> {
    let pairs = FlowchartParser::parse(Rule::diagram, input)
        .map_err(|e| MermaidError::ParseError(e.to_string()))?;

    for pair in pairs {
        match pair.as_rule() {
            Rule::diagram => {
                for inner in pair.into_inner() {
                    process_rule(inner, db)?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn process_rule(pair: pest::iterators::Pair<Rule>, db: &mut FlowchartDb) -> Result<()> {
    match pair.as_rule() {
        Rule::graph_config => {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::direction {
                    db.set_direction(inner.as_str());
                }
            }
        }
        Rule::document => {
            for inner in pair.into_inner() {
                process_rule(inner, db)?;
            }
        }
        Rule::statement => {
            for inner in pair.into_inner() {
                process_rule(inner, db)?;
            }
        }
        Rule::direction_stmt => {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::direction {
                    db.set_direction(inner.as_str());
                }
            }
        }
        Rule::vertex_statement => {
            process_vertex_statement(pair, db)?;
        }
        Rule::acc_title_stmt => {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::line_content {
                    db.set_acc_title(inner.as_str().trim());
                }
            }
        }
        Rule::acc_descr_stmt => {
            process_acc_descr(pair, db)?;
        }
        Rule::class_def_stmt => {
            process_class_def(pair, db)?;
        }
        Rule::class_stmt => {
            process_class_stmt(pair, db)?;
        }
        Rule::style_stmt => {
            process_style_stmt(pair, db)?;
        }
        Rule::link_style_stmt => {
            process_link_style_stmt(pair, db)?;
        }
        Rule::click_stmt => {
            process_click_stmt(pair, db)?;
        }
        Rule::subgraph_stmt => {
            process_subgraph(pair, db)?;
        }
        _ => {}
    }
    Ok(())
}

fn process_vertex_statement(pair: pest::iterators::Pair<Rule>, db: &mut FlowchartDb) -> Result<()> {
    let mut nodes: Vec<String> = Vec::new();
    let mut pending_link: Option<(String, Option<String>, Option<String>)> = None; // (arrow, text, id)

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::node => {
                let node_ids = process_node(inner, db)?;

                // If we have a pending link, connect previous nodes to current nodes
                if let Some((arrow, text, link_id)) = pending_link.take() {
                    for from in &nodes {
                        for to in &node_ids {
                            db.add_edge(from, to, &arrow, text.as_deref(), link_id.as_deref());
                        }
                    }
                }

                nodes = node_ids;
            }
            Rule::link => {
                pending_link = Some(process_link(inner)?);
            }
            _ => {}
        }
    }

    Ok(())
}

fn process_node(pair: pest::iterators::Pair<Rule>, db: &mut FlowchartDb) -> Result<Vec<String>> {
    let mut node_ids = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::styled_vertex => {
                let (id, class) = process_styled_vertex(inner, db)?;
                if let Some(class_name) = class {
                    db.set_class(&id, &class_name);
                }
                node_ids.push(id);
            }
            Rule::node_group => {
                for styled in inner.into_inner() {
                    if styled.as_rule() == Rule::styled_vertex {
                        let (id, class) = process_styled_vertex(styled, db)?;
                        if let Some(class_name) = class {
                            db.set_class(&id, &class_name);
                        }
                        node_ids.push(id);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(node_ids)
}

fn process_styled_vertex(pair: pest::iterators::Pair<Rule>, db: &mut FlowchartDb) -> Result<(String, Option<String>)> {
    let mut vertex_id = String::new();
    let mut class_name = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::vertex => {
                vertex_id = process_vertex(inner, db)?;
            }
            Rule::class_name => {
                class_name = Some(inner.as_str().to_string());
            }
            _ => {}
        }
    }

    Ok((vertex_id, class_name))
}

fn process_vertex(pair: pest::iterators::Pair<Rule>, db: &mut FlowchartDb) -> Result<String> {
    let mut id = String::new();
    let mut text: Option<FlowText> = None;
    let mut vertex_type: Option<FlowVertexType> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                id = inner.as_str().to_string();
            }
            Rule::vertex_shape => {
                let (shape, shape_text) = process_vertex_shape(inner)?;
                vertex_type = Some(shape);
                text = Some(shape_text);
            }
            _ => {}
        }
    }

    db.add_vertex_simple(&id, text.as_ref().map(|t| t.text.as_str()), vertex_type);
    Ok(id)
}

fn process_vertex_shape(pair: pest::iterators::Pair<Rule>) -> Result<(FlowVertexType, FlowText)> {
    let inner = pair.into_inner().next().unwrap();

    let (shape_type, text) = match inner.as_rule() {
        Rule::shape_square => (FlowVertexType::Square, extract_text(inner)?),
        Rule::shape_round => (FlowVertexType::Round, extract_text(inner)?),
        Rule::shape_circle => (FlowVertexType::Circle, extract_text(inner)?),
        Rule::shape_double_circle => (FlowVertexType::DoubleCircle, extract_text(inner)?),
        Rule::shape_stadium => (FlowVertexType::Stadium, extract_text(inner)?),
        Rule::shape_subroutine => (FlowVertexType::Subroutine, extract_text(inner)?),
        Rule::shape_cylinder => (FlowVertexType::Cylinder, extract_text(inner)?),
        Rule::shape_diamond => (FlowVertexType::Diamond, extract_text(inner)?),
        Rule::shape_hexagon => (FlowVertexType::Hexagon, extract_text(inner)?),
        Rule::shape_ellipse => (FlowVertexType::Ellipse, extract_text(inner)?),
        Rule::shape_odd => (FlowVertexType::Odd, extract_text(inner)?),
        Rule::shape_trapezoid => (FlowVertexType::Trapezoid, extract_text(inner)?),
        Rule::shape_inv_trapezoid => (FlowVertexType::InvTrapezoid, extract_text(inner)?),
        Rule::shape_lean_right => (FlowVertexType::LeanRight, extract_text(inner)?),
        Rule::shape_lean_left => (FlowVertexType::LeanLeft, extract_text(inner)?),
        _ => return Err(MermaidError::ParseError("Unknown shape type".to_string())),
    };

    Ok((shape_type, text))
}

fn extract_text(pair: pest::iterators::Pair<Rule>) -> Result<FlowText> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::text {
            return process_text(inner);
        }
    }
    Ok(FlowText::new(""))
}

fn process_text(pair: pest::iterators::Pair<Rule>) -> Result<FlowText> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::quoted_string => {
                let s = inner.as_str();
                // Remove surrounding quotes
                let text = &s[1..s.len() - 1];
                return Ok(FlowText::new(text));
            }
            Rule::md_string => {
                let s = inner.as_str();
                // Remove surrounding "`...`"
                let text = &s[2..s.len() - 2];
                return Ok(FlowText::markdown(text));
            }
            Rule::plain_text => {
                return Ok(FlowText::new(inner.as_str().trim()));
            }
            _ => {}
        }
    }
    Ok(FlowText::new(""))
}

fn process_link(pair: pest::iterators::Pair<Rule>) -> Result<(String, Option<String>, Option<String>)> {
    let mut arrow = String::new();
    let mut text = None;
    let mut link_id = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::simple_link => {
                for link_inner in inner.into_inner() {
                    match link_inner.as_rule() {
                        Rule::link_arrow => {
                            arrow = link_inner.as_str().to_string();
                        }
                        Rule::link_id => {
                            let id_str = link_inner.as_str();
                            link_id = Some(id_str[..id_str.len() - 1].to_string()); // Remove @
                        }
                        _ => {}
                    }
                }
            }
            Rule::link_with_text => {
                for link_inner in inner.into_inner() {
                    match link_inner.as_rule() {
                        Rule::link_start => {
                            arrow = link_inner.as_str().to_string();
                        }
                        Rule::link_end => {
                            arrow.push_str(link_inner.as_str());
                        }
                        Rule::link_arrow => {
                            arrow = link_inner.as_str().to_string();
                        }
                        Rule::edge_text => {
                            text = Some(link_inner.as_str().trim().to_string());
                        }
                        Rule::text => {
                            let flow_text = process_text(link_inner)?;
                            text = Some(flow_text.text);
                        }
                        Rule::link_id => {
                            let id_str = link_inner.as_str();
                            link_id = Some(id_str[..id_str.len() - 1].to_string());
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok((arrow, text, link_id))
}

fn process_acc_descr(pair: pest::iterators::Pair<Rule>, db: &mut FlowchartDb) -> Result<()> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::acc_descr_single => {
                for i in inner.into_inner() {
                    if i.as_rule() == Rule::line_content {
                        db.set_acc_description(i.as_str().trim());
                    }
                }
            }
            Rule::acc_descr_multi => {
                for i in inner.into_inner() {
                    if i.as_rule() == Rule::multiline_content {
                        db.set_acc_description(i.as_str().trim());
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn process_class_def(pair: pest::iterators::Pair<Rule>, db: &mut FlowchartDb) -> Result<()> {
    let mut identifier = String::new();
    let mut styles_str = String::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::class_id => {
                identifier = inner.as_str().to_string();
            }
            Rule::styles => {
                styles_str = inner.as_str().trim().to_string();
            }
            _ => {}
        }
    }

    // Split styles by comma
    let styles: Vec<String> = styles_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    db.add_class(&identifier, &styles);
    Ok(())
}

fn process_class_stmt(pair: pest::iterators::Pair<Rule>, db: &mut FlowchartDb) -> Result<()> {
    let mut identifiers = Vec::new();
    let mut class_name = String::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier_list => {
                for id in inner.into_inner() {
                    if id.as_rule() == Rule::identifier {
                        identifiers.push(id.as_str().to_string());
                    }
                }
            }
            Rule::identifier => {
                class_name = inner.as_str().to_string();
            }
            _ => {}
        }
    }

    for id in identifiers {
        db.set_class(&id, &class_name);
    }
    Ok(())
}

fn process_style_stmt(pair: pest::iterators::Pair<Rule>, db: &mut FlowchartDb) -> Result<()> {
    let mut identifier = String::new();
    let mut styles_str = String::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                identifier = inner.as_str().to_string();
            }
            Rule::styles => {
                styles_str = inner.as_str().trim().to_string();
            }
            _ => {}
        }
    }

    let styles: Vec<String> = styles_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if let Some(vertex) = db.get_vertex_mut(&identifier) {
        vertex.styles = styles;
    }
    Ok(())
}

fn process_link_style_stmt(pair: pest::iterators::Pair<Rule>, db: &mut FlowchartDb) -> Result<()> {
    let mut indices = Vec::new();
    let mut styles_str = String::new();
    let mut interpolate = None;
    let mut is_default = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::num_list => {
                for num in inner.into_inner() {
                    if num.as_rule() == Rule::NUMBER {
                        if let Ok(n) = num.as_str().parse::<usize>() {
                            indices.push(n);
                        }
                    }
                }
            }
            Rule::identifier => {
                // Could be "default" or interpolate value
                let id = inner.as_str();
                if id == "default" {
                    is_default = true;
                } else {
                    interpolate = Some(id.to_string());
                }
            }
            Rule::styles => {
                styles_str = inner.as_str().trim().to_string();
            }
            _ => {}
        }
    }

    let styles: Vec<String> = styles_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if is_default {
        db.set_default_link_style(&styles);
    } else {
        for idx in indices {
            db.set_link_style(idx, &styles);
        }
    }

    if let Some(interp) = interpolate {
        db.set_default_link_interpolate(&interp);
    }

    Ok(())
}

fn process_click_stmt(pair: pest::iterators::Pair<Rule>, db: &mut FlowchartDb) -> Result<()> {
    let mut node_id = String::new();
    let mut callback = None;
    let mut href = None;
    let mut tooltip = None;
    let mut link_target = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                node_id = inner.as_str().to_string();
            }
            Rule::click_action => {
                for action in inner.into_inner() {
                    match action.as_rule() {
                        Rule::callback_action | Rule::simple_callback => {
                            for a in action.into_inner() {
                                match a.as_rule() {
                                    Rule::identifier => {
                                        callback = Some(a.as_str().to_string());
                                    }
                                    Rule::tooltip => {
                                        for t in a.into_inner() {
                                            if t.as_rule() == Rule::quoted_string {
                                                let s = t.as_str();
                                                tooltip = Some(s[1..s.len()-1].to_string());
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Rule::href_action => {
                            for a in action.into_inner() {
                                match a.as_rule() {
                                    Rule::quoted_string => {
                                        let s = a.as_str();
                                        href = Some(s[1..s.len()-1].to_string());
                                    }
                                    Rule::link_target => {
                                        for t in a.into_inner() {
                                            link_target = Some(t.as_str().to_string());
                                        }
                                    }
                                    Rule::tooltip => {
                                        for t in a.into_inner() {
                                            if t.as_rule() == Rule::quoted_string {
                                                let s = t.as_str();
                                                tooltip = Some(s[1..s.len()-1].to_string());
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(cb) = callback {
        db.set_click_event(&node_id, &cb);
    }
    if let Some(h) = href {
        db.set_link(&node_id, &h, link_target.as_deref());
    }
    if let Some(t) = tooltip {
        db.set_tooltip(&node_id, &t);
    }

    Ok(())
}

fn process_subgraph(pair: pest::iterators::Pair<Rule>, db: &mut FlowchartDb) -> Result<()> {
    let mut id = String::new();
    let mut title = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::subgraph_id => {
                for i in inner.into_inner() {
                    match i.as_rule() {
                        Rule::identifier => {
                            id = i.as_str().to_string();
                        }
                        Rule::text => {
                            let flow_text = process_text(i)?;
                            title = Some(flow_text.text);
                        }
                        _ => {}
                    }
                }
            }
            Rule::document => {
                // Recursively process subgraph content
                for doc_inner in inner.into_inner() {
                    process_rule(doc_inner, db)?;
                }
            }
            _ => {}
        }
    }

    db.add_subgraph(&id, title.as_deref().unwrap_or(&id));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_flowchart() {
        let input = "flowchart LR\n    A --> B";
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let db = result.unwrap();
        assert!(db.get_vertices().contains_key("A"));
        assert!(db.get_vertices().contains_key("B"));
        assert_eq!(db.get_edges().len(), 1);
    }

    #[test]
    fn test_parse_with_labels() {
        let input = "flowchart TD\n    A[Start] --> B[End]";
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let db = result.unwrap();
        let a = db.get_vertices().get("A").unwrap();
        assert_eq!(a.text, Some("Start".to_string()));
    }

    #[test]
    fn test_parse_different_shapes() {
        let input = r#"flowchart LR
    A[Square]
    B(Round)
    C((Circle))
    D{Diamond}
    E[(Cylinder)]
    F([Stadium])"#;
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let db = result.unwrap();
        assert_eq!(db.get_vertices().len(), 6);
    }

    #[test]
    fn test_parse_edge_with_text() {
        let input = "flowchart LR\n    A -->|label| B";
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let db = result.unwrap();
        let edges = db.get_edges();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].text, "label");
    }

    #[test]
    fn test_parse_style_stmt() {
        // Style statements need the vertex to exist first
        let input = "flowchart LR\nA[Start]\nstyle A fill:#f9f";
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let db = result.unwrap();
        let vertex = db.get_vertices().get("A").unwrap();
        assert!(vertex.styles.len() > 0, "Vertex A should have styles: {:?}", vertex);
    }

    #[test]
    fn test_parse_class_def() {
        let input = "flowchart LR\nclassDef myClass fill:#f9f";
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let db = result.unwrap();
        assert!(db.get_classes().contains_key("myClass"), "myClass should be defined");
    }

    #[test]
    fn test_parse_subgraph() {
        let input = r#"flowchart LR
subgraph sub1[Title]
    A --> B
end"#;
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let db = result.unwrap();
        assert!(!db.subgraphs().is_empty(), "Should have subgraphs");
    }

    #[test]
    fn test_parse_direction_stmt() {
        let input = "flowchart LR\ndirection TB";
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let db = result.unwrap();
        assert_eq!(db.get_direction(), "TB", "Direction should be changed to TB");
    }

    #[test]
    fn test_parse_click_stmt() {
        let input = r#"flowchart LR
A[Node]
click A myCallback"#;
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
    }

    #[test]
    fn test_parse_link_style() {
        let input = "flowchart LR\nA --> B\nlinkStyle 0 stroke:#ff0";
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
    }

    #[test]
    fn test_parse_node_with_class() {
        let input = "flowchart LR\nA:::myClass --> B";
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let db = result.unwrap();
        let vertex = db.get_vertices().get("A").expect("Vertex A should exist");
        assert!(vertex.classes.contains(&"myClass".to_string()));
    }

    #[test]
    fn test_parse_multiple_edges() {
        let input = "flowchart LR\nA --> B --> C";
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let db = result.unwrap();
        assert_eq!(db.get_edges().len(), 2);
    }

    #[test]
    fn test_parse_graph_keyword() {
        let input = "graph TD\nA --> B";
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse with 'graph' keyword: {:?}", result);
    }

    #[test]
    fn test_parse_thick_arrow() {
        let input = "flowchart LR\nA ==> B";
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse thick arrow: {:?}", result);
        let db = result.unwrap();
        assert_eq!(db.get_edges().len(), 1);
    }

    #[test]
    fn test_parse_dotted_arrow() {
        let input = "flowchart LR\nA -.-> B";
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse dotted arrow: {:?}", result);
        let db = result.unwrap();
        assert_eq!(db.get_edges().len(), 1);
    }

    #[test]
    fn test_parse_hexagon_shape() {
        let input = "flowchart LR\nA{{Hexagon}}";
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse hexagon: {:?}", result);
        let db = result.unwrap();
        let vertex = db.get_vertices().get("A").unwrap();
        assert_eq!(vertex.vertex_type, Some(FlowVertexType::Hexagon));
    }

    #[test]
    fn test_parse_acc_title() {
        let input = "flowchart LR\naccTitle: My Chart Title\nA --> B";
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let db = result.unwrap();
        assert_eq!(db.get_acc_title(), Some("My Chart Title"));
    }

    #[test]
    fn test_parse_quoted_text() {
        let input = r#"flowchart LR
A["Node with spaces"]"#;
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse quoted text: {:?}", result);
        let db = result.unwrap();
        let vertex = db.get_vertices().get("A").unwrap();
        assert_eq!(vertex.text, Some("Node with spaces".to_string()));
    }

    #[test]
    fn test_parse_semicolon_newline() {
        let input = "flowchart LR;A --> B;C --> D";
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse semicolon-separated: {:?}", result);
        let db = result.unwrap();
        assert_eq!(db.get_edges().len(), 2);
    }

    #[test]
    fn test_parse_bidirectional_arrow() {
        let input = "flowchart LR\nA <--> B";
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse bidirectional: {:?}", result);
    }

    #[test]
    fn test_parse_node_group() {
        let input = "flowchart LR\nA & B --> C";
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse node group: {:?}", result);
        let db = result.unwrap();
        // Should have edges from both A and B to C
        assert_eq!(db.get_edges().len(), 2);
    }

    #[test]
    fn test_parse_nested_subgraph() {
        let input = r#"flowchart LR
subgraph outer
    subgraph inner
        A --> B
    end
end"#;
        let result = parse(input);
        assert!(result.is_ok(), "Failed to parse nested subgraph: {:?}", result);
        let db = result.unwrap();
        assert_eq!(db.subgraphs().len(), 2);
    }
}
