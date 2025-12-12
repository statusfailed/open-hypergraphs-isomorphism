/// Isomorphisms of open hypergraphs
pub mod types;

/// Constraints on nodes and edges
pub mod constraint;

// Michael McLeod's traversal-based algorithm for isomorphism finding in monogamous hypergraphs
pub mod traversal;

/// "propagator" algorithm (WIP) for finding isomorphisms in non-monogamous connected open
/// hypergraphs
pub mod propagator;

pub use types::Isomorphism;
