//! CRDT realization: e = (−⊔x), S = join-closed sublattice, γ = contraction rate.
//!
//! In CRDTs, the idempotent is the "merge" or "join" operation applied to convergence.
//! The solution space is a join-closed sublattice — once you've seen all values,
//! further merges are no-ops.
//! The contraction rate is how fast replicas converge.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A CRDT value — a grow-only counter (GCounter) for simplicity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GCounter {
    pub counts: BTreeMap<String, u64>,
}

impl GCounter {
    pub fn new() -> Self {
        Self { counts: BTreeMap::new() }
    }

    pub fn increment(&mut self, node: &str, amount: u64) {
        *self.counts.entry(node.to_string()).or_insert(0) += amount;
    }

    /// The join (merge) operation — component-wise max.
    pub fn join(&self, other: &Self) -> Self {
        let mut result = self.counts.clone();
        for (k, &v) in &other.counts {
            result.entry(k.clone()).and_modify(|existing| {
                *existing = (*existing).max(v);
            }).or_insert(v);
        }
        Self { counts: result }
    }

    /// The idempotent: merge with all known replicas' values.
    /// e(x) = x ⊔ x = x (idempotent by construction).
    pub fn idempotent_apply(&self) -> Self {
        self.clone() // Join with self = self
    }

    pub fn total(&self) -> u64 {
        self.counts.values().sum()
    }
}

/// A GSet (grow-only set) CRDT.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GSet<T: Ord + Clone> {
    pub elements: BTreeSet<T>,
}

impl<T: Ord + Clone> GSet<T> {
    pub fn new() -> Self {
        Self { elements: BTreeSet::new() }
    }

    pub fn insert(&mut self, val: T) {
        self.elements.insert(val);
    }

    /// Join = union.
    pub fn join(&self, other: &Self) -> Self {
        Self {
            elements: self.elements.union(&other.elements).cloned().collect(),
        }
    }

    pub fn contains(&self, val: &T) -> bool {
        self.elements.contains(val)
    }
}

/// An LWW-Register (Last-Writer-Wins) CRDT.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LwwRegister<T: Clone + PartialEq> {
    pub value: T,
    pub timestamp: u64,
}

impl<T: Clone + PartialEq> LwwRegister<T> {
    pub fn new(value: T, timestamp: u64) -> Self {
        Self { value, timestamp }
    }

    /// Join = take the one with higher timestamp.
    pub fn join(&self, other: &Self) -> Self {
        if self.timestamp >= other.timestamp {
            self.clone()
        } else {
            other.clone()
        }
    }

    /// Idempotent: joining with self gives self.
    pub fn idempotent_apply(&self) -> Self {
        self.clone()
    }
}

/// CRDT Realization of constitutive computation.
///
/// The idempotent e = merge/reconciliation.
/// The solution space S = fully converged state (join-closed sublattice).
/// The spectral gap γ = contraction rate of the merge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtRealization {
    pub label: String,
    pub node_count: usize,
    /// Contraction rate (how fast replicas converge per round).
    pub contraction_rate: f64,
    /// Current round of gossip.
    pub current_round: u64,
    /// Whether the state is fully converged.
    pub is_converged: bool,
}

impl CrdtRealization {
    pub fn new(label: &str, node_count: usize, contraction_rate: f64) -> Self {
        Self {
            label: label.to_string(),
            node_count,
            contraction_rate,
            current_round: 0,
            is_converged: false,
        }
    }

    /// Simulate one round of gossip merging.
    /// Returns the fraction of nodes not yet converged.
    pub fn gossip_round(&mut self, unconverged_fraction: f64) -> f64 {
        self.current_round += 1;
        let remaining = unconverged_fraction * (-self.contraction_rate).exp();
        self.is_converged = remaining < 1e-10;
        remaining
    }

    /// Compute convergence bound after t rounds.
    pub fn convergence_bound(&self, t: u64) -> f64 {
        (-self.contraction_rate * t as f64).exp()
    }

    /// Zero-cost: if the CRDT state is already converged, further merges are no-ops.
    pub fn zero_cost_check(&self, state: &GCounter, all_known: &GCounter) -> bool {
        let merged = state.join(all_known);
        merged == *all_known // Already have everything
    }

    /// Verify that join is idempotent, commutative, and associative.
    pub fn verify_crdt_laws(a: &GCounter, b: &GCounter, c: &GCounter) -> CrdtLawVerification {
        // Idempotent: a ⊔ a = a
        let idempotent = a.join(a) == *a;

        // Commutative: a ⊔ b = b ⊔ a
        let commutative = a.join(b) == b.join(a);

        // Associative: (a ⊔ b) ⊔ c = a ⊔ (b ⊔ c)
        let associative = a.join(&b.join(c)) == a.join(b).join(c);

        CrdtLawVerification {
            idempotent,
            commutative,
            associative,
            is_valid_crdt: idempotent && commutative && associative,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtLawVerification {
    pub idempotent: bool,
    pub commutative: bool,
    pub associative: bool,
    pub is_valid_crdt: bool,
}

/// Verify GSet CRDT laws.
impl<T: Ord + Clone + Debug> GSet<T> {
    pub fn verify_laws(a: &GSet<T>, b: &GSet<T>, c: &GSet<T>) -> CrdtLawVerification {
        let idempotent = a.join(a) == *a;
        let commutative = a.join(b) == b.join(a);
        let associative = a.join(&b.join(c)) == a.join(b).join(c);

        CrdtLawVerification {
            idempotent,
            commutative,
            associative,
            is_valid_crdt: idempotent && commutative && associative,
        }
    }
}

use std::fmt::Debug;

/// Verify LWW Register CRDT laws.
impl<T: Clone + PartialEq + Debug> LwwRegister<T> {
    pub fn verify_laws(a: &LwwRegister<T>, b: &LwwRegister<T>) -> bool {
        // Idempotent
        let idempotent = a.idempotent_apply() == *a;
        // Commutative
        let commutative = a.join(b) == b.join(a);
        idempotent && commutative
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcounter_idempotent() {
        let mut a = GCounter::new();
        a.increment("node1", 5);
        a.increment("node2", 3);
        assert_eq!(a.join(&a), a, "Join with self = self");
    }

    #[test]
    fn test_gcounter_commutative() {
        let mut a = GCounter::new();
        a.increment("node1", 5);
        let mut b = GCounter::new();
        b.increment("node1", 3);
        b.increment("node2", 7);
        assert_eq!(a.join(&b), b.join(&a));
    }

    #[test]
    fn test_gcounter_associative() {
        let mut a = GCounter::new();
        a.increment("n1", 1);
        let mut b = GCounter::new();
        b.increment("n2", 2);
        let mut c = GCounter::new();
        c.increment("n1", 5);
        c.increment("n3", 3);
        assert_eq!(a.join(&b.join(&c)), a.join(&b).join(&c));
    }

    #[test]
    fn test_gcounter_crdt_laws() {
        let mut a = GCounter::new();
        a.increment("n1", 5);
        let mut b = GCounter::new();
        b.increment("n1", 3);
        b.increment("n2", 7);
        let mut c = GCounter::new();
        c.increment("n3", 2);
        let v = CrdtRealization::verify_crdt_laws(&a, &b, &c);
        assert!(v.is_valid_crdt);
    }

    #[test]
    fn test_crdt_convergence() {
        let mut crdt = CrdtRealization::new("gcounter", 3, 1.0);
        let mut fraction = 1.0;
        for _ in 0..10 {
            fraction = crdt.gossip_round(fraction);
        }
        assert!(fraction < 0.001, "Should converge after 10 rounds, got {}", fraction);
    }

    #[test]
    fn test_crdt_zero_cost_converged() {
        let mut all = GCounter::new();
        all.increment("n1", 5);
        all.increment("n2", 3);
        let crdt = CrdtRealization::new("gc", 2, 1.0);
        assert!(crdt.zero_cost_check(&all, &all));
    }

    #[test]
    fn test_crdt_nonzero_cost_unconverged() {
        let mut state = GCounter::new();
        state.increment("n1", 5);
        let mut all = GCounter::new();
        all.increment("n1", 5);
        all.increment("n2", 3);
        let crdt = CrdtRealization::new("gc", 2, 1.0);
        assert!(!crdt.zero_cost_check(&state, &all));
    }

    #[test]
    fn test_gset_idempotent() {
        let mut a = GSet::new();
        a.insert(1);
        a.insert(2);
        assert_eq!(a.join(&a), a);
    }

    #[test]
    fn test_gset_commutative() {
        let mut a = GSet::new();
        a.insert(1);
        let mut b = GSet::new();
        b.insert(2);
        assert_eq!(a.join(&b), b.join(&a));
        let merged = a.join(&b);
        assert!(merged.contains(&1));
        assert!(merged.contains(&2));
    }

    #[test]
    fn test_gset_laws() {
        let mut a: GSet<i32> = GSet::new();
        a.insert(1);
        let mut b: GSet<i32> = GSet::new();
        b.insert(2);
        let mut c: GSet<i32> = GSet::new();
        c.insert(3);
        let v = GSet::verify_laws(&a, &b, &c);
        assert!(v.is_valid_crdt);
    }

    #[test]
    fn test_lww_idempotent() {
        let reg = LwwRegister::new("hello", 100);
        assert_eq!(reg.idempotent_apply(), reg);
    }

    #[test]
    fn test_lww_last_writer_wins() {
        let a = LwwRegister::new("old", 1);
        let b = LwwRegister::new("new", 2);
        let merged = a.join(&b);
        assert_eq!(merged.value, "new");
        assert_eq!(merged.timestamp, 2);
    }

    #[test]
    fn test_lww_laws() {
        let a = LwwRegister::new("a", 1);
        let b = LwwRegister::new("b", 2);
        assert!(LwwRegister::verify_laws(&a, &b));
    }

    #[test]
    fn test_convergence_bound() {
        let crdt = CrdtRealization::new("test", 5, 2.0);
        assert!(crdt.convergence_bound(0) > 0.99);
        assert!(crdt.convergence_bound(5) < 1e-4);
    }
}
