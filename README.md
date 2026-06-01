# lau-constitutive-compute

**Constitutive Computation** — the Zero-Cost Theorem, Karoubi splitting, and Naturality Boundary for structure-as-computation.

---

## What This Does

This crate implements the mathematical framework for systems where **structure pays the bill**. It provides:

- **Core abstractions**: idempotent endomorphisms, Karoubi splitting (A→S→A), dynamical realizations, and conservation-by-construction
- **Hodge realization**: discrete Laplacian → harmonic projection, spectral gap as convergence rate
- **CRDT realization**: merge/join as idempotent, join-closed sublattices as solution spaces
- **Symplectic realization**: moment-map reduction, constraint surfaces, energy conservation
- **Warp realization**: distributed consensus (Paxos-style) as idempotent commit, agreed predicates as solution
- **Kolmogorov residue**: estimating K(answer | structure) as the irreducible runtime cost
- **Agent design**: skills as inclusions i: S ↪ A, optimal behavior as fixed points of contraction maps

The crate contains **73 tests** verifying all mathematical properties.

---

## Key Idea

> *A computation is eliminable exactly to the extent that its answer is NATURAL — determined uniformly by structure for all instances.*

The framework rests on three pillars:

1. **Zero-Cost Theorem**: If constructors factor through the inclusion i: S ↪ A, then solving costs zero. The type system IS the proof.
2. **Karoubi Splitting**: An idempotent e: A→A (e∘e=e) splits as A—r→S—i→A with r∘i=id_S, i∘r=e. The solution space S is a genuine mathematical object.
3. **Naturality Boundary**: The irreducible runtime residue equals K(answer | structure) — the Kolmogorov complexity of the answer given the structure. Natural computations have zero residue.

---

## Install

```toml
[dependencies]
lau-constitutive-compute = { git = "https://github.com/SuperInstance/lau-constitutive-compute" }
```

### Dependencies

- `serde` 1.x (with `derive`)
- `nalgebra` 0.33 (with `serde-serialize`)

---

## Quick Start

```rust
use lau_constitutive_compute::{
    KaroubiSplitting, DynamicalRealization, ZeroCostTheorem,
    NaturalityBoundary, HodgeRealization,
    hodge::path_laplacian,
};

// Naturality Boundary — zero residue means structure fully determines answer
let natural = NaturalityBoundary::natural();
assert!(natural.is_natural);
assert_eq!(natural.residue_bits, 0.0);

// Dynamical realization — exponential convergence
let dyn_real = DynamicalRealization::new("my-system", 2.0, "generator L");
assert!(dyn_real.transient_cost_bound(1.0) < 0.14);  // e^{-2}
let time = dyn_real.time_to_tolerance(0.001);

// Hodge decomposition on a path graph
let laplacian = path_laplacian(5);
let hodge = HodgeRealization::from_laplacian(laplacian);
assert!(hodge.verify_idempotence());
```

Run all 73 tests:

```bash
cargo test
```

---

## API Reference

### Core (`core`)

| Type | Description |
|------|-------------|
| `KaroubiSplitting<A, S>` | Splitting A—r→S—i→A verifying r∘i=id_S, i∘r=e |
| `SplittingVerification` | Result of checking splitting axioms on test points |
| `DynamicalRealization` | Semigroup φ_t → e with generator L and spectral gap γ |
| `ZeroCostTheorem` | Check: if state is in S, computation costs zero |
| `NaturalityBoundary` | Residue = K(answer\|structure); is_natural when residue = 0 |
| `ConstitutiveAnalysis` | Complete analysis: idempotent, gap, boundary, category |
| `ConservationByConstruction<A, S>` | Type-system guarantees: only constructable states are in S |
| `ComputationCategory` | `Hodge` \| `CRDT` \| `Symplectic` \| `Warp` |

### Hodge Realization (`hodge`)

The Hodge decomposition: any form = exact + coexact + harmonic. The idempotent e = I − ΔG projects away the harmonic component.

| Function/Method | Description |
|------|-------------|
| `HodgeRealization::from_laplacian(Δ)` | Build from a discrete Laplacian |
| `.project(v)` | Apply e: project onto non-harmonic component |
| `.harmonic_projection(v)` | Extract harmonic component |
| `.verify_idempotence()` | Check e² = e |
| `.verify_kernel_projection()` | Check e projects onto ker(Δ) |
| `.is_harmonic(v)` | Test if v ∈ ker(Δ) |
| `.zero_cost_check(v)` | Harmonic vectors stay harmonic under projection |
| `.transient_cost(t)` | e^{-γt} bound |
| `path_laplacian(n)` | Laplacian of a path graph (no harmonic forms) |
| `cycle_laplacian(n)` | Laplacian of a cycle graph (1 harmonic form: constant) |
| `complete_laplacian(n)` | Laplacian of a complete graph (1 harmonic form) |

### CRDT Realization (`crdt`)

CRDTs (Conflict-free Replicated Data Types): the merge/join operation is idempotent, commutative, and associative.

| Type | Description |
|------|-------------|
| `GCounter` | Grow-only counter (component-wise max) |
| `GSet<T>` | Grow-only set (union) |
| `LwwRegister<T>` | Last-writer-wins register (max timestamp) |
| `CrdtRealization` | Simulation of gossip convergence with contraction rate |
| `CrdtLawVerification` | Verified idempotent + commutative + associative |

### Symplectic Realization (`symplectic`)

Hamiltonian mechanics: the idempotent is moment-map reduction to a constraint surface.

| Type/Method | Description |
|------|-------------|
| `SymplecticSystem` | Coupled oscillators: H = ½pᵀM⁻¹p + ½qᵀKq |
| `.hamiltonian(q, p)` | Total energy |
| `.equations_of_motion(q, p)` | Returns (dq/dt, dp/dt) |
| `.step(q, p, dt)` | Symplectic Euler integrator (preserves symplectic form) |
| `.moment_map(q, p)` | Conserved quantity μ = Σqᵢpᵢ |
| `.idempotent_project(q, p, target)` | Project onto μ⁻¹(target) |
| `SymplecticRealization` | Wraps system + target moment + spectral gap |
| `.verify_idempotence(q, p)` | Projecting twice = projecting once |
| `.verify_energy_conservation(q, p, steps, dt)` | H stays approximately constant |

### Warp Realization (`warp`)

Distributed consensus (Paxos-style): the idempotent is the commit process.

| Type/Method | Description |
|------|-------------|
| `Ballot(seq, node)` | Ballot number for consensus |
| `WarpMessage<T>` | Prepare / Promise / Accept / Accepted / Decided |
| `AcceptorState<T>` | Single acceptor's state (promised, accepted, decided) |
| `WarpRealization<T>` | Multi-acceptor consensus with quorum |
| `.run_consensus(ballot, value)` | Full Phase 1 + Phase 2 + Decide |
| `.idempotent_check(value)` | Re-proposing decided value returns same |
| `.zero_cost_check()` | All decided same value → no messages needed |
| `.spectral_gap()` | 1 / latency_bound |

### Kolmogorov Residue (`kolmogorov`)

Estimating K(answer | structure) — the uncomputable but upper-bounded irreducible cost.

| Function | Description |
|------|-------------|
| `KolmogorovResidue::estimate_complexity(data)` | RLE-based compression estimate |
| `::estimate_conditional(answer, structure)` | K(answer\|structure) approximation |
| `::analyze(answer, structure, desc)` | Full analysis: k_answer, k_structure, k_conditional, naturality |
| `structure_complexity(type)` | Reference complexity for known structure types |

### Agent Design (`agent`)

Agents designed through constitutive computation: skills as inclusions, behavior as fixed points.

| Type/Method | Description |
|------|-------------|
| `Skill` | A skill with name, solution_space, spectral_gap, is_zero_cost |
| `AgentDesign` | Agent with skills and contraction rate |
| `.add_skill(skill)` | Add a skill (inclusion i: S ↪ A) |
| `.compute_fixed_point(state)` | Iterate contraction T until convergence |
| `.analyze()` | AgentAnalysis: skills, gaps, naturality ratio |
| `AgentState` | Mutable state with knowledge map and cost tracking |

---

## How It Works

### The Four Realizations

The crate instantiates the constitutive computation framework in four domains:

| Domain | Idempotent e | Solution Space S | Spectral Gap γ |
|--------|-------------|-----------------|----------------|
| **Hodge** | I − ΔG (harmonic projection) | Harmonic forms ker(Δ) | Smallest nonzero eigenvalue of Δ |
| **CRDT** | Merge/join (⊔) | Join-closed sublattice | Contraction rate of gossip |
| **Symplectic** | Moment-map reduction | μ⁻¹(c)/G (Marsden-Weinstein) | Stability of reduced system |
| **Warp** | Commit (ballot→decide) | Agreed predicate | 1/latency |

Each realizes the same abstract structure: an idempotent that determines a solution space, with a spectral gap governing convergence.

### Dynamical Realization

The physics IS the relaxation. A system evolves under generator L toward the fixed point e. The spectral gap γ determines how fast:

```
transient_cost(t) ≤ e^{-γt}
```

For states born inside the solution space S (via the inclusion i), the cost is exactly zero — there is no transient to decay.

### Karoubi Splitting

Given an idempotent e: A → A, the Karoubi envelope produces:

```
A —r→ S —i→ A
```

where r∘i = id_S and i∘r = e. This makes the solution space S a first-class object, not just a property of A. The retraction r projects onto the solution; the inclusion i embeds it back.

### Conservation by Construction

When all public constructors go through the inclusion i: S ↪ A, the type system guarantees that every constructable value is in the solution space. You cannot build a non-solution state through the public API. This is conservation-by-construction: the type system IS the proof of the Zero-Cost Theorem.

---

## The Math

### Idempotents and the Karoubi Envelope

An **idempotent** is a map e: A → A satisfying e∘e = e. The image im(e) = {e(a) | a ∈ A} is the set of fixed points. In the Karoubi envelope (a.k.a. idempotent completion), every idempotent splits:

- r: A → S (retraction onto the image)
- i: S → A (inclusion of the image)
- r∘i = id_S, i∘r = e

### The Zero-Cost Theorem

If a value x ∈ A is constructed via the inclusion i (i.e., x = i(s) for some s ∈ S), then:

```
e(x) = (i∘r)∘(i(s)) = i∘(r∘i)(s) = i(s) = x
```

The computation costs nothing because there's nothing to compute.

### The Naturality Boundary

For any computation that takes structure σ and produces answer α:

```
runtime_residue ≥ K(α | σ)
```

where K is Kolmogorov complexity. If the answer is fully determined by the structure (natural in the category-theoretic sense), then K(α|σ) = 0 and the computation is entirely eliminable.

### Hodge Theory on Graphs

For a graph Laplacian Δ with eigenvalues λ₁ ≤ λ₂ ≤ ... ≤ λₙ:

- Harmonic space = ker(Δ) = span of eigenvectors with λᵢ = 0
- Spectral gap = smallest λᵢ > 0
- Green's function G = Δ⁺ (pseudoinverse)
- Idempotent e = I − ΔG projects onto non-harmonic component

The spectral gap determines the Poincaré inequality constant and the convergence rate of diffusion/heat flow on the graph.

### Symplectic Mechanics

For Hamiltonian H = ½pᵀM⁻¹p + ½qᵀKq:

- Equations of motion: dq/dt = ∂H/∂p = M⁻¹p, dp/dt = −∂H/∂q = −Kq
- Symplectic form ω = Σ dpᵢ ∧ dqᵢ is preserved along trajectories
- Moment map μ(q,p) = Σqᵢpᵢ gives conserved quantities
- Reduction to μ⁻¹(c) is the idempotent: project onto constraint surface

### CRDT Semilattices

CRDTs live on join-semilattices (S, ⊔) where ⊔ is:

- Idempotent: a ⊔ a = a
- Commutative: a ⊔ b = b ⊔ a
- Associative: (a ⊔ b) ⊔ c = a ⊔ (b ⊔ c)

The merge operation IS the idempotent. Convergence is guaranteed because the semilattice has a unique top element reachable from any starting state.

---

## License

MIT
