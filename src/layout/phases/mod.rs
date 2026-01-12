//! Layout algorithm phases
//!
//! The layered layout algorithm (Sugiyama method) runs in 5 phases:
//! 1. Cycle removal - remove cycles to make the graph acyclic
//! 2. Layer assignment - assign nodes to layers (ranks)
//! 3. Crossing minimization - reorder nodes within layers
//! 4. Node positioning - assign x,y coordinates
//! 5. Edge routing - compute edge paths with bend points

mod cycle_removal;
mod layering;
mod ordering;
mod positioning;
mod routing;

pub use cycle_removal::remove_cycles;
pub use layering::assign_layers;
pub use ordering::minimize_crossings;
pub use positioning::position_nodes;
pub use routing::route_edges;
