//! Diagram type detection

use regex::Regex;
use std::sync::LazyLock;

use crate::error::{MermaidError, Result};

/// Supported diagram types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagramType {
    Flowchart,
    Info,
    Mindmap,
    Pie,
}

// Regex patterns for detecting diagram types
static FLOWCHART_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\s*(flowchart|graph)\s*(TB|BT|RL|LR|TD)?").unwrap()
});

static PIE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^\s*pie").unwrap());

static MINDMAP_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^\s*mindmap").unwrap());

static INFO_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^\s*info").unwrap());

/// Detect the type of diagram from input text
pub fn detect_type(input: &str) -> Result<DiagramType> {
    // Remove comments and frontmatter
    let cleaned = remove_frontmatter(input);
    let cleaned = remove_comments(&cleaned);

    if PIE_RE.is_match(&cleaned) {
        return Ok(DiagramType::Pie);
    }

    if MINDMAP_RE.is_match(&cleaned) {
        return Ok(DiagramType::Mindmap);
    }

    if INFO_RE.is_match(&cleaned) {
        return Ok(DiagramType::Info);
    }

    if FLOWCHART_RE.is_match(&cleaned) {
        return Ok(DiagramType::Flowchart);
    }

    Err(MermaidError::UnknownDiagramType(
        "Could not detect diagram type".to_string(),
    ))
}

/// Remove YAML frontmatter from input
fn remove_frontmatter(input: &str) -> String {
    let trimmed = input.trim_start();
    if trimmed.starts_with("---") {
        if let Some(end_pos) = trimmed[3..].find("---") {
            return trimmed[end_pos + 6..].to_string();
        }
    }
    input.to_string()
}

/// Remove comments from input
fn remove_comments(input: &str) -> String {
    input
        .lines()
        .filter(|line| !line.trim().starts_with("%%"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_flowchart() {
        assert_eq!(
            detect_type("flowchart LR\n  A --> B").unwrap(),
            DiagramType::Flowchart
        );
        assert_eq!(
            detect_type("graph TD\n  A --> B").unwrap(),
            DiagramType::Flowchart
        );
        assert_eq!(
            detect_type("  flowchart TB\n  A --> B").unwrap(),
            DiagramType::Flowchart
        );
    }

    #[test]
    fn detect_pie() {
        assert_eq!(detect_type("pie\n  \"A\": 50").unwrap(), DiagramType::Pie);
        assert_eq!(detect_type("  pie\n  \"A\": 50").unwrap(), DiagramType::Pie);
        assert_eq!(
            detect_type("pie showData\n  \"A\": 50").unwrap(),
            DiagramType::Pie
        );
    }

    #[test]
    fn detect_mindmap() {
        assert_eq!(
            detect_type("mindmap\n  root").unwrap(),
            DiagramType::Mindmap
        );
        assert_eq!(
            detect_type("  mindmap\n  root").unwrap(),
            DiagramType::Mindmap
        );
    }

    #[test]
    fn detect_info() {
        assert_eq!(detect_type("info").unwrap(), DiagramType::Info);
        assert_eq!(detect_type("info showInfo").unwrap(), DiagramType::Info);
        assert_eq!(detect_type("  info").unwrap(), DiagramType::Info);
    }

    #[test]
    fn detect_with_comments() {
        assert_eq!(
            detect_type("%% this is a comment\nflowchart LR\n  A --> B").unwrap(),
            DiagramType::Flowchart
        );
    }

    #[test]
    fn unknown_type() {
        assert!(detect_type("unknown diagram").is_err());
    }
}
