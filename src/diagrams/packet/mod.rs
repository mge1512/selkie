//! Packet diagram support
//!
//! This module provides data structures for packet diagrams.
//! Packet diagrams show bit-level packet/protocol structure with labeled fields.

mod types;
pub mod parser;

pub use types::{PacketBlock, PacketDb, PacketError, PacketWord};
pub use parser::parse;
