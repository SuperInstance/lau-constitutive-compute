//! # Constitutive Computation
//!
//! The mathematical framework for systems where structure pays the bill.
//!
//! **Core theorem:** A computation is eliminable exactly to the extent that its
//! answer is NATURAL — determined uniformly by structure for all instances.
//!
//! **Zero-Cost Theorem:** If constructors factor through the inclusion i: S ↪ A,
//! then solving costs zero.
//!
//! **Naturality Boundary:** The irreducible runtime residue equals K(answer | structure)
//! — the Kolmogorov complexity of the answer given the structure.

mod core;
mod hodge;
mod crdt;
mod symplectic;
mod warp;
mod kolmogorov;
mod agent;

pub use core::*;
pub use hodge::HodgeRealization;
pub use crdt::CrdtRealization;
pub use symplectic::SymplecticRealization;
pub use warp::WarpRealization;
pub use kolmogorov::KolmogorovResidue;
pub use agent::AgentDesign;
