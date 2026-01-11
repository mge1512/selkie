//! Flowchart parser
//!
//! This module provides parsing for flowchart diagrams.
//! The actual grammar parsing will be implemented using pest.

use super::types::FlowchartDb;
use crate::error::Result;

/// Parse a flowchart diagram
pub fn parse(input: &str) -> Result<FlowchartDb> {
    let mut db = FlowchartDb::new();
    parse_into(input, &mut db)?;
    Ok(db)
}

/// Parse into an existing database
pub fn parse_into(_input: &str, _db: &mut FlowchartDb) -> Result<()> {
    // TODO: Implement actual parsing using pest grammar
    // For now, this is a placeholder that will be filled in
    // as we implement the grammar
    Ok(())
}
