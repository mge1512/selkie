//! Size estimation for layout

use super::adapter::{NodeSizeConfig, SizeEstimator};
use super::types::NodeShape;

/// Character-width based size estimator
///
/// This estimator uses average character widths to approximate text dimensions
/// without requiring a rendering context. It's suitable for layout purposes
/// where exact pixel-perfect sizing isn't critical.
#[derive(Debug, Clone)]
pub struct CharacterSizeEstimator {
    /// Average character width ratio (relative to font size)
    pub char_width_ratio: f64,
    /// Line height ratio (relative to font size)
    pub line_height_ratio: f64,
}

impl Default for CharacterSizeEstimator {
    fn default() -> Self {
        Self {
            // Approximate ratio for proportional fonts like Verdana/Arial
            char_width_ratio: 0.6,
            line_height_ratio: 1.4,
        }
    }
}

impl CharacterSizeEstimator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an estimator optimized for monospace fonts
    pub fn monospace() -> Self {
        Self {
            char_width_ratio: 0.6,
            line_height_ratio: 1.2,
        }
    }
}

impl SizeEstimator for CharacterSizeEstimator {
    fn estimate_text_size(&self, text: &str, font_size: f64) -> (f64, f64) {
        if text.is_empty() {
            return (0.0, font_size * self.line_height_ratio);
        }

        let lines: Vec<&str> = text.lines().collect();
        let max_chars = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let num_lines = lines.len().max(1);

        let width = (max_chars as f64) * font_size * self.char_width_ratio;
        let height = (num_lines as f64) * font_size * self.line_height_ratio;

        (width, height)
    }

    fn estimate_node_size(
        &self,
        label: Option<&str>,
        shape: NodeShape,
        config: &NodeSizeConfig,
    ) -> (f64, f64) {
        // Calculate text dimensions
        let (text_width, text_height) = label
            .map(|l| self.estimate_text_size(l, config.font_size))
            .unwrap_or((0.0, 0.0));

        // Add padding
        let base_width = text_width + config.padding_horizontal * 2.0;
        let base_height = text_height + config.padding_vertical * 2.0;

        // Apply shape-specific adjustments
        let (width, height) = match shape {
            NodeShape::Circle | NodeShape::DoubleCircle => {
                // Circle needs diameter = max(width, height)
                let diameter = base_width.max(base_height);
                (diameter, diameter)
            }
            NodeShape::Diamond => {
                // Diamond is a square rotated 45 degrees, matching mermaid.js:
                // size = width + height (not scaling, but sum)
                // This ensures the text fits inside the rotated square
                let s = base_width + base_height;
                (s, s)
            }
            NodeShape::Hexagon => {
                // Hexagon needs extra horizontal space for angled sides
                (base_width * 1.2, base_height)
            }
            NodeShape::Ellipse => {
                // Ellipse needs slightly more space
                (base_width * 1.1, base_height * 1.1)
            }
            NodeShape::Stadium => {
                // Stadium (pill shape) needs extra width for rounded ends
                (base_width + base_height, base_height)
            }
            NodeShape::Cylinder => {
                // Cylinder needs extra height for 3D cap
                (base_width, base_height * 1.3)
            }
            NodeShape::Trapezoid | NodeShape::InvTrapezoid => {
                // Trapezoid needs extra width for angled sides
                (base_width * 1.2, base_height)
            }
            NodeShape::LeanRight | NodeShape::LeanLeft => {
                // Parallelogram needs extra width
                (base_width * 1.2, base_height)
            }
            NodeShape::Subroutine => {
                // Subroutine has extra side bars
                (base_width + 20.0, base_height)
            }
            NodeShape::Odd => {
                // Odd shape (flag-like) - asymmetric
                (base_width * 1.1, base_height)
            }
            NodeShape::Rectangle | NodeShape::RoundedRect => {
                // Standard rectangles - no adjustment needed
                (base_width, base_height)
            }
        };

        // Apply min/max constraints
        let final_width = width.max(config.min_width);
        let final_height = height.max(config.min_height);
        let final_width = config
            .max_width
            .map(|max| final_width.min(max))
            .unwrap_or(final_width);

        (final_width, final_height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_size_estimation() {
        let estimator = CharacterSizeEstimator::default();

        let (w, h) = estimator.estimate_text_size("Hello", 14.0);
        assert!(w > 0.0);
        assert!(h > 0.0);

        // Longer text should be wider
        let (w2, _) = estimator.estimate_text_size("Hello World", 14.0);
        assert!(w2 > w);

        // Multiline text should be taller
        let (_, h2) = estimator.estimate_text_size("Line1\nLine2", 14.0);
        assert!(h2 > h);
    }

    #[test]
    fn test_node_size_with_shapes() {
        let estimator = CharacterSizeEstimator::default();
        let config = NodeSizeConfig::default();

        let (rect_w, rect_h) = estimator.estimate_node_size(
            Some("Test"),
            NodeShape::Rectangle,
            &config,
        );

        // Diamond should be larger than rectangle for same text
        let (diamond_w, diamond_h) = estimator.estimate_node_size(
            Some("Test"),
            NodeShape::Diamond,
            &config,
        );
        assert!(diamond_w > rect_w);
        assert!(diamond_h > rect_h);

        // Circle should have equal width and height
        let (circle_w, circle_h) = estimator.estimate_node_size(
            Some("Test"),
            NodeShape::Circle,
            &config,
        );
        assert!((circle_w - circle_h).abs() < 0.001);
    }

    #[test]
    fn test_min_size_constraints() {
        let estimator = CharacterSizeEstimator::default();
        let config = NodeSizeConfig {
            min_width: 100.0,
            min_height: 50.0,
            ..Default::default()
        };

        // Even with no label, should meet minimum size
        let (w, h) = estimator.estimate_node_size(None, NodeShape::Rectangle, &config);
        assert!(w >= 100.0);
        assert!(h >= 50.0);
    }
}
