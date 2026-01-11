//! Flowchart types

use std::collections::HashMap;

use crate::common::CommonDb;

/// Valid vertex types in flowcharts
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FlowVertexType {
    #[default]
    Square,
    DoubleCircle,
    Circle,
    Ellipse,
    Stadium,
    Subroutine,
    Rect,
    Cylinder,
    Round,
    Diamond,
    Hexagon,
    Odd,
    Trapezoid,
    InvTrapezoid,
    LeanRight,
    LeanLeft,
}

impl FlowVertexType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "square" => Some(Self::Square),
            "doublecircle" => Some(Self::DoubleCircle),
            "circle" => Some(Self::Circle),
            "ellipse" => Some(Self::Ellipse),
            "stadium" => Some(Self::Stadium),
            "subroutine" => Some(Self::Subroutine),
            "rect" => Some(Self::Rect),
            "cylinder" => Some(Self::Cylinder),
            "round" => Some(Self::Round),
            "diamond" => Some(Self::Diamond),
            "hexagon" => Some(Self::Hexagon),
            "odd" => Some(Self::Odd),
            "trapezoid" => Some(Self::Trapezoid),
            "inv_trapezoid" => Some(Self::InvTrapezoid),
            "lean_right" => Some(Self::LeanRight),
            "lean_left" => Some(Self::LeanLeft),
            _ => None,
        }
    }
}

/// Text type for flowchart labels
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FlowTextType {
    #[default]
    Text,
    Markdown,
}

/// Text content for a flowchart element
#[derive(Debug, Clone, Default)]
pub struct FlowText {
    pub text: String,
    pub text_type: FlowTextType,
}

/// A vertex (node) in a flowchart
#[derive(Debug, Clone)]
pub struct FlowVertex {
    pub id: String,
    pub dom_id: String,
    pub text: Option<String>,
    pub label_type: FlowTextType,
    pub vertex_type: Option<FlowVertexType>,
    pub styles: Vec<String>,
    pub classes: Vec<String>,
    pub dir: Option<String>,
    pub link: Option<String>,
    pub link_target: Option<String>,
    pub have_callback: bool,
    pub icon: Option<String>,
    pub form: Option<String>,
    pub pos: Option<String>,
    pub img: Option<String>,
    pub constraint: Option<String>,
}

impl FlowVertex {
    pub fn new(id: impl Into<String>, dom_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            dom_id: dom_id.into(),
            text: None,
            label_type: FlowTextType::Text,
            vertex_type: None,
            styles: Vec::new(),
            classes: Vec::new(),
            dir: None,
            link: None,
            link_target: None,
            have_callback: false,
            icon: None,
            form: None,
            pos: None,
            img: None,
            constraint: None,
        }
    }
}

/// Edge stroke types
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum EdgeStroke {
    #[default]
    Normal,
    Thick,
    Invisible,
    Dotted,
}

/// An edge (link) between nodes in a flowchart
#[derive(Debug, Clone)]
pub struct FlowEdge {
    pub id: Option<String>,
    pub is_user_defined_id: bool,
    pub start: String,
    pub end: String,
    pub interpolate: Option<String>,
    pub edge_type: Option<String>,
    pub stroke: EdgeStroke,
    pub style: Vec<String>,
    pub length: Option<u32>,
    pub text: String,
    pub label_type: FlowTextType,
    pub classes: Vec<String>,
    pub animation: Option<String>,
    pub animate: Option<bool>,
}

impl FlowEdge {
    pub fn new(start: impl Into<String>, end: impl Into<String>) -> Self {
        Self {
            id: None,
            is_user_defined_id: false,
            start: start.into(),
            end: end.into(),
            interpolate: None,
            edge_type: None,
            stroke: EdgeStroke::Normal,
            style: Vec::new(),
            length: None,
            text: String::new(),
            label_type: FlowTextType::Text,
            classes: Vec::new(),
            animation: None,
            animate: None,
        }
    }
}

/// A class definition for styling
#[derive(Debug, Clone, Default)]
pub struct FlowClass {
    pub id: String,
    pub styles: Vec<String>,
    pub text_styles: Vec<String>,
}

/// A subgraph (container for nodes)
#[derive(Debug, Clone, Default)]
pub struct FlowSubGraph {
    pub id: String,
    pub title: String,
    pub label_type: String,
    pub nodes: Vec<String>,
    pub classes: Vec<String>,
    pub dir: Option<String>,
}

/// Link type information from parser
#[derive(Debug, Clone, Default)]
pub struct FlowLink {
    pub link_type: Option<String>,
    pub stroke: EdgeStroke,
    pub length: Option<u32>,
    pub text: Option<FlowText>,
    pub id: Option<String>,
}

/// Data returned from getData()
#[derive(Debug, Clone)]
pub struct FlowData {
    pub vertices: HashMap<String, FlowVertex>,
    pub edges: Vec<FlowEdge>,
    pub classes: HashMap<String, FlowClass>,
    pub subgraphs: Vec<FlowSubGraph>,
}

/// The flowchart database
#[derive(Debug, Clone)]
pub struct FlowchartDb {
    /// Common database fields
    common: CommonDb,
    /// Counter for generating unique vertex IDs
    vertex_counter: u32,
    /// All vertices in the chart
    vertices: HashMap<String, FlowVertex>,
    /// All edges (with optional default interpolate)
    edges: Vec<FlowEdge>,
    /// Default interpolation for edges
    default_interpolate: Option<String>,
    /// Default style for edges
    default_style: Option<Vec<String>>,
    /// CSS class definitions
    classes: HashMap<String, FlowClass>,
    /// Subgraphs
    subgraphs: Vec<FlowSubGraph>,
    /// Subgraph lookup by ID
    subgraph_lookup: HashMap<String, usize>,
    /// Tooltips for elements
    tooltips: HashMap<String, String>,
    /// Direction of the flowchart
    direction: String,
}

impl Default for FlowchartDb {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowchartDb {
    const DOM_ID_PREFIX: &'static str = "flowchart-";

    pub fn new() -> Self {
        Self {
            common: CommonDb::new(),
            vertex_counter: 0,
            vertices: HashMap::new(),
            edges: Vec::new(),
            default_interpolate: None,
            default_style: None,
            classes: HashMap::new(),
            subgraphs: Vec::new(),
            subgraph_lookup: HashMap::new(),
            tooltips: HashMap::new(),
            direction: String::from("TB"),
        }
    }

    pub fn clear(&mut self) {
        self.common.clear();
        self.vertex_counter = 0;
        self.vertices.clear();
        self.edges.clear();
        self.default_interpolate = None;
        self.default_style = None;
        self.classes.clear();
        self.subgraphs.clear();
        self.subgraph_lookup.clear();
        self.tooltips.clear();
        self.direction = String::from("TB");
    }

    /// Check if a node exists in any of the given subgraphs
    pub fn exists(&self, subgraphs: &[FlowSubGraph], node_id: &str) -> bool {
        subgraphs.iter().any(|sg| sg.nodes.contains(&node_id.to_string()))
    }

    /// Remove nodes from a subgraph that already exist in other subgraphs
    pub fn make_uniq(&self, subgraph: &mut FlowSubGraph, existing: &[FlowSubGraph]) {
        subgraph.nodes.retain(|node| !self.exists(existing, node));
    }

    /// Add a vertex to the flowchart
    pub fn add_vertex(
        &mut self,
        id: &str,
        text_obj: Option<FlowText>,
        vertex_type: Option<FlowVertexType>,
        styles: Vec<String>,
        classes: Vec<String>,
        dir: Option<&str>,
        _metadata: Option<&str>,
    ) {
        if id.trim().is_empty() {
            return;
        }

        let vertex = self.vertices.entry(id.to_string()).or_insert_with(|| {
            let dom_id = format!("{}{}-{}", Self::DOM_ID_PREFIX, id, self.vertex_counter);
            FlowVertex::new(id, dom_id)
        });

        self.vertex_counter += 1;

        if let Some(text_obj) = text_obj {
            let mut txt = text_obj.text.trim().to_string();
            // Strip quotes if string starts and ends with a quote
            if txt.starts_with('"') && txt.ends_with('"') && txt.len() > 1 {
                txt = txt[1..txt.len() - 1].to_string();
            }
            vertex.text = Some(txt);
            vertex.label_type = text_obj.text_type;
        } else if vertex.text.is_none() {
            vertex.text = Some(id.to_string());
        }

        if let Some(vt) = vertex_type {
            vertex.vertex_type = Some(vt);
        }

        vertex.styles.extend(styles);
        vertex.classes.extend(classes);

        if let Some(d) = dir {
            vertex.dir = Some(d.to_string());
        }
    }

    /// Add an edge between nodes
    fn add_single_link(&mut self, start: &str, end: &str, link_data: Option<&FlowLink>, id: Option<&str>) {
        let mut edge = FlowEdge::new(start, end);
        edge.interpolate = self.default_interpolate.clone();

        if let Some(link) = link_data {
            if let Some(text) = &link.text {
                let mut txt = text.text.trim().to_string();
                if txt.starts_with('"') && txt.ends_with('"') && txt.len() > 1 {
                    txt = txt[1..txt.len() - 1].to_string();
                }
                edge.text = txt;
                edge.label_type = text.text_type.clone();
            }
            edge.edge_type = link.link_type.clone();
            edge.stroke = link.stroke.clone();
            if let Some(len) = link.length {
                edge.length = Some(len.min(10));
            }
        }

        if let Some(user_id) = id {
            if !self.edges.iter().any(|e| e.id.as_deref() == Some(user_id)) {
                edge.id = Some(user_id.to_string());
                edge.is_user_defined_id = true;
            }
        }

        if edge.id.is_none() {
            let existing_count = self.edges.iter().filter(|e| e.start == edge.start && e.end == edge.end).count();
            edge.id = Some(format!("L-{}-{}-{}", start, end, existing_count));
        }

        self.edges.push(edge);
    }

    /// Add links between multiple start and end nodes
    pub fn add_link(&mut self, starts: &[&str], ends: &[&str], link_data: Option<&FlowLink>) {
        let id = link_data.and_then(|l| l.id.as_deref());

        for (si, start) in starts.iter().enumerate() {
            for (ei, end) in ends.iter().enumerate() {
                // Only use ID for last start and first end
                let use_id = si == starts.len() - 1 && ei == 0;
                self.add_single_link(start, end, link_data, if use_id { id } else { None });
            }
        }
    }

    /// Update link interpolation
    pub fn update_link_interpolate(&mut self, positions: &[String], interpolate: &str) {
        for pos in positions {
            if pos == "default" {
                self.default_interpolate = Some(interpolate.to_string());
                // Apply to existing edges that don't have explicit interpolate
                for edge in &mut self.edges {
                    if edge.interpolate.is_none() {
                        edge.interpolate = Some(interpolate.to_string());
                    }
                }
            } else if let Ok(idx) = pos.parse::<usize>() {
                if idx < self.edges.len() {
                    self.edges[idx].interpolate = Some(interpolate.to_string());
                }
            }
        }
    }

    /// Update link style
    pub fn update_link(&mut self, positions: &[usize], style: &[String]) {
        for &pos in positions {
            if pos == usize::MAX {
                // "default" case
                self.default_style = Some(style.to_vec());
            } else if pos < self.edges.len() {
                self.edges[pos].style = style.to_vec();
                // If fill not set, add fill:none
                if !self.edges[pos].style.iter().any(|s| s.starts_with("fill")) {
                    self.edges[pos].style.push("fill:none".to_string());
                }
            }
        }
    }

    /// Add a CSS class definition
    pub fn add_class(&mut self, ids: &str, styles: &[String]) {
        // Process styles: join, handle escaped commas, split by semicolons
        let processed_style: Vec<String> = styles
            .join(",")
            .replace("\\,", "§§§")
            .replace(',', ";")
            .replace("§§§", ",")
            .split(';')
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect();

        for id in ids.split(',') {
            let id = id.trim();
            let class_node = self.classes.entry(id.to_string()).or_insert_with(|| FlowClass {
                id: id.to_string(),
                styles: Vec::new(),
                text_styles: Vec::new(),
            });

            for style in &processed_style {
                if style.contains("color") {
                    let new_style = style.replace("fill", "bgFill");
                    class_node.text_styles.push(new_style);
                }
                class_node.styles.push(style.clone());
            }
        }
    }

    /// Set the direction of the flowchart
    pub fn set_direction(&mut self, dir: &str) {
        let mut direction = dir.trim().to_string();

        if direction.contains('<') {
            direction = "RL".to_string();
        } else if direction.contains('^') {
            direction = "BT".to_string();
        } else if direction.contains('>') {
            direction = "LR".to_string();
        } else if direction.contains('v') {
            direction = "TB".to_string();
        } else if direction == "TD" {
            direction = "TB".to_string();
        }

        self.direction = direction;
    }

    /// Get the direction of the flowchart
    pub fn get_direction(&self) -> &str {
        &self.direction
    }

    /// Set class on elements
    pub fn set_class(&mut self, ids: &str, class_name: &str) {
        for id in ids.split(',') {
            let id = id.trim();
            if let Some(vertex) = self.vertices.get_mut(id) {
                vertex.classes.push(class_name.to_string());
            }
            for edge in &mut self.edges {
                if edge.id.as_deref() == Some(id) {
                    edge.classes.push(class_name.to_string());
                }
            }
            if let Some(&idx) = self.subgraph_lookup.get(id) {
                if idx < self.subgraphs.len() {
                    self.subgraphs[idx].classes.push(class_name.to_string());
                }
            }
        }
    }

    /// Add a subgraph
    pub fn add_sub_graph(&mut self, nodes: Vec<String>, id: &str, title: &str, dir: &str) {
        let subgraph = FlowSubGraph {
            id: id.to_string(),
            title: title.to_string(),
            label_type: "text".to_string(),
            nodes,
            classes: Vec::new(),
            dir: if dir.is_empty() { None } else { Some(dir.to_string()) },
        };

        let idx = self.subgraphs.len();
        self.subgraph_lookup.insert(id.to_string(), idx);
        self.subgraphs.push(subgraph);
    }

    /// Get all vertices
    pub fn get_vertices(&self) -> &HashMap<String, FlowVertex> {
        &self.vertices
    }

    /// Get all edges
    pub fn get_edges(&self) -> &[FlowEdge] {
        &self.edges
    }

    /// Get all classes
    pub fn get_classes(&self) -> &HashMap<String, FlowClass> {
        &self.classes
    }

    /// Get all subgraphs
    pub fn get_subgraphs(&self) -> &[FlowSubGraph] {
        &self.subgraphs
    }

    /// Get data for rendering
    pub fn get_data(&self) -> FlowData {
        FlowData {
            vertices: self.vertices.clone(),
            edges: self.edges.clone(),
            classes: self.classes.clone(),
            subgraphs: self.subgraphs.clone(),
        }
    }

    // Common DB passthrough methods
    pub fn set_acc_title(&mut self, title: impl Into<String>) {
        self.common.set_acc_title(title);
    }

    pub fn get_acc_title(&self) -> Option<&str> {
        self.common.get_acc_title()
    }

    pub fn set_acc_description(&mut self, desc: impl Into<String>) {
        self.common.set_acc_description(desc);
    }

    pub fn get_acc_description(&self) -> Option<&str> {
        self.common.get_acc_description()
    }

    pub fn set_diagram_title(&mut self, title: impl Into<String>) {
        self.common.set_diagram_title(title);
    }

    pub fn get_diagram_title(&self) -> Option<&str> {
        self.common.get_diagram_title()
    }
}
