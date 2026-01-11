//! Requirement diagram support
//!
//! This module provides data structures for requirement diagrams.
//! Requirement diagrams show requirements, elements, and their relationships.

mod types;
pub mod parser;

pub use types::{
    Element, Relation, RelationshipType, Requirement, RequirementClass, RequirementDb,
    RequirementType, RiskLevel, VerifyType,
};
pub use parser::parse;
