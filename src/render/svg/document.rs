//! SVG document builder

use super::elements::SvgElement;

/// SVG document builder
#[derive(Debug, Clone)]
pub struct SvgDocument {
    /// Document width
    width: f64,
    /// Document height
    height: f64,
    /// View box (minX, minY, width, height)
    view_box: Option<(f64, f64, f64, f64)>,
    /// Style content
    styles: Vec<String>,
    /// Definition elements (markers, gradients, etc.)
    defs: Vec<SvgElement>,
    /// Content elements
    elements: Vec<SvgElement>,
}

impl SvgDocument {
    pub fn new() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            view_box: None,
            styles: Vec::new(),
            defs: Vec::new(),
            elements: Vec::new(),
        }
    }

    /// Set the document size
    pub fn set_size(&mut self, width: f64, height: f64) {
        self.width = width;
        self.height = height;
        self.view_box = Some((0.0, 0.0, width, height));
    }

    /// Set the document size with custom viewBox origin
    /// Use this when content has negative coordinates
    pub fn set_size_with_origin(&mut self, min_x: f64, min_y: f64, width: f64, height: f64) {
        self.width = width;
        self.height = height;
        self.view_box = Some((min_x, min_y, width, height));
    }

    /// Add a style block
    pub fn add_style(&mut self, css: &str) {
        self.styles.push(css.to_string());
    }

    /// Add definition elements (markers, etc.)
    pub fn add_defs(&mut self, elements: Vec<SvgElement>) {
        self.defs.extend(elements);
    }

    /// Add a content element
    pub fn add_element(&mut self, element: SvgElement) {
        self.elements.push(element);
    }

    /// Convert to SVG string
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        // XML declaration
        result.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");

        // SVG opening tag
        let view_box_str = self
            .view_box
            .map(|(x, y, w, h)| format!(" viewBox=\"{} {} {} {}\"", x, y, w, h))
            .unwrap_or_default();

        result.push_str(&format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\"{} class=\"mermaid\">\n",
            self.width, self.height, view_box_str
        ));

        // Styles
        if !self.styles.is_empty() {
            result.push_str("  <style>\n");
            for style in &self.styles {
                result.push_str(style);
                result.push('\n');
            }
            result.push_str("  </style>\n");
        }

        // Defs
        if !self.defs.is_empty() {
            result.push_str("  <defs>\n");
            for def in &self.defs {
                result.push_str(&def.to_svg(2));
                result.push('\n');
            }
            result.push_str("  </defs>\n");
        }

        // Content group
        result.push_str("  <g class=\"root\">\n");

        // Elements
        for element in &self.elements {
            result.push_str(&element.to_svg(2));
            result.push('\n');
        }

        result.push_str("  </g>\n");
        result.push_str("</svg>\n");

        result
    }
}

impl Default for SvgDocument {
    fn default() -> Self {
        Self::new()
    }
}
