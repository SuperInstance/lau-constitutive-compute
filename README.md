# lau-constitutive-compute

> Constitutive Computation — the Zero-Cost Theorem, Karoubi splitting, and Naturality Boundary for structure-as-computation

## What This Does

Constitutive Computation — the Zero-Cost Theorem, Karoubi splitting, and Naturality Boundary for structure-as-computation. Part of the PLATO/LAU ecosystem — a mathematically rigorous framework for building educational agents that learn, teach, and evolve.

## The Key Idea

This crate implements the core abstractions needed for its domain, with a focus on correctness, composability, and conservation guarantees. Every public type is serializable (serde), every algorithm is tested, and every invariant is verified.

## Install

```bash
cargo add lau-constitutive-compute
```

## Quick Start

See the API Reference below for complete usage. Key entry points:

```rust
use lau_constitutive_compute::*;
// See types and methods below for complete usage
```

## API Reference

```rust
pub struct Idempotent<A: Clone + Debug + PartialEq> 
pub struct KaroubiSplitting<A: Clone + Debug + PartialEq, S: Clone + Debug + PartialEq> 
    pub fn new(ambient_label: &str, solution_label: &str, spectral_gap: f64) -> Self 
    pub fn verify_splitting(
pub struct SplittingVerification 
pub struct DynamicalRealization 
    pub fn new(label: &str, spectral_gap: f64, generator_description: &str) -> Self 
    pub fn transient_cost_bound(&self, t: f64) -> f64 
    pub fn time_to_tolerance(&self, epsilon: f64) -> f64 
pub struct ZeroCostTheorem 
    pub fn check(is_in_solution: bool, dynamical: &DynamicalRealization, t: f64) -> Self 
pub struct NaturalityBoundary 
    pub fn new(residue_bits: f64, description: &str) -> Self 
    pub fn natural() -> Self 
    pub fn eliminability(&self, total_complexity_bits: f64) -> f64 
pub struct ConstitutiveAnalysis 
pub enum ComputationCategory 
pub struct ConservationByConstruction<A, S> 
    pub fn construct(&self, s: &S) -> A 
    pub fn is_conserved(&self, a: &A) -> bool 
    pub fn verify_conservation(&self, test_values: &[S]) -> bool 
pub struct KolmogorovResidue 
    pub fn estimate_complexity(data: &[u8]) -> f64 
    pub fn estimate_conditional(answer: &[u8], structure: &[u8]) -> f64 
    pub fn analyze(answer: &[u8], structure: &[u8], description: &str) -> Self 
    pub fn fully_natural(description: &str) -> Self 
pub fn structure_complexity(structure_type: &str) -> f64 
pub fn naturality_boundary(
pub struct Skill 
    pub fn new(name: &str, solution_space: &str, spectral_gap: f64, description: &str) -> Self 
pub struct AgentDesign 
    pub fn new(name: &str, contraction_rate: f64) -> Self 
    pub fn add_skill(&mut self, skill: Skill) 
    pub fn compute_fixed_point(&self, initial_state: &AgentState) -> AgentState 
    pub fn analyze(&self) -> AgentAnalysis 
    pub fn find_skill_idempotent(&self, skill_name: &str) -> Option<SkillIdempotent> 
pub struct AgentState 
    pub fn new() -> Self 
    pub fn distance(&self, other: &Self) -> f64 
pub struct AgentAnalysis 
pub struct SkillIdempotent 
pub fn analyze_system(
pub struct Ballot(pub u64, pub String);
pub enum WarpMessage<T: Clone + PartialEq> 
pub struct AcceptorState<T: Clone + PartialEq> 
    pub fn new(node_id: &str) -> Self 
    pub fn handle_prepare(&mut self, ballot: &Ballot) -> Option<WarpMessage<T>> 
    pub fn handle_accept(&mut self, ballot: &Ballot, value: &T) -> Option<WarpMessage<T>> 
    pub fn decide(&mut self, value: &T) 
    pub fn is_decided(&self) -> bool 
pub struct WarpRealization<T: Clone + PartialEq> 
    pub fn new(node_ids: &[&str], latency_bound_ms: f64) -> Self 
    pub fn run_consensus(&mut self, ballot: Ballot, value: T) -> WarpConsensusResult<T> 
    pub fn idempotent_check(&self, decided_value: &T) -> bool 
    pub fn zero_cost_check(&self) -> bool 
    pub fn agreed_value(&self) -> Option<T> 
    pub fn spectral_gap(&self) -> f64 
    pub fn rounds_to_converge(&self, reliability: f64) -> f64 
pub enum WarpConsensusResult<T: Clone + PartialEq> 
pub struct HodgeRealization 
```

## How It Works

Read the source in `src/` for full implementation details. All algorithms are documented with inline comments explaining the mathematical foundations.

## The Math

This crate implements formal mathematical constructs. See the source documentation for theorem statements and proofs of correctness.

## Testing

**73 tests** covering construction, serialization, correctness properties, edge cases, and composability with other lau-* crates.

## License

MIT
