//! Core abstractions: idempotent, Karoubi splitting, dynamical realization,
//! Zero-Cost Theorem, Naturality Boundary.

use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// An idempotent endomorphism e: A → A with e∘e = e.
///
/// The image of e IS the solution space. Everything in the image is already solved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Idempotent<A: Clone + Debug + PartialEq> {
    /// The carrier set (or type) — the ambient space A.
    pub carrier_label: String,
    /// Apply the idempotent to a value.
    /// We store a closure as a function pointer won't work generically,
    /// so instead we use the `solve` method on the realization.
    pub description: String,
    _phantom: std::marker::PhantomData<A>,
}

/// Karoubi splitting: A—r→S—i→A with r∘i=id_S, i∘r=e.
///
/// This says the solution space S is a genuine object, not just a property.
/// The retraction r projects onto the solution; the inclusion i embeds it back.
/// The idempotent e = i∘r is the "solver".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KaroubiSplitting<A: Clone + Debug + PartialEq, S: Clone + Debug + PartialEq> {
    pub ambient_label: String,
    pub solution_label: String,
    pub spectral_gap: f64,
    _phantom: std::marker::PhantomData<(A, S)>,
}

impl<A: Clone + Debug + PartialEq, S: Clone + Debug + PartialEq> KaroubiSplitting<A, S> {
    /// Create a new Karoubi splitting.
    pub fn new(ambient_label: &str, solution_label: &str, spectral_gap: f64) -> Self {
        Self {
            ambient_label: ambient_label.to_string(),
            solution_label: solution_label.to_string(),
            spectral_gap,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Verify the splitting axioms: r∘i = id_S and i∘r = e.
    /// Takes actual functions and checks the conditions.
    pub fn verify_splitting(
        &self,
        r: &dyn Fn(&A) -> S,
        i: &dyn Fn(&S) -> A,
        e: &dyn Fn(&A) -> A,
        test_points: &[S],
        ambient_test: &[A],
    ) -> SplittingVerification {
        let mut ri_ok = true;
        let mut ir_ok = true;

        // r∘i = id_S
        for s in test_points {
            let ai = i(s);
            let ri_s = r(&ai);
            if *s != ri_s {
                ri_ok = false;
                break;
            }
        }

        // i∘r = e
        for a in ambient_test {
            let ir_a = i(&r(a));
            let e_a = e(a);
            if ir_a != e_a {
                ir_ok = false;
                break;
            }
        }

        SplittingVerification {
            ri_is_id: ri_ok,
            ir_is_e: ir_ok,
            is_valid: ri_ok && ir_ok,
        }
    }
}

/// Result of verifying a Karoubi splitting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplittingVerification {
    pub ri_is_id: bool,
    pub ir_is_e: bool,
    pub is_valid: bool,
}

/// Dynamical realization: a semigroup φ_t → e with generator L.
///
/// The physics IS the relaxation. The system evolves toward the fixed point
/// which is the solution. The spectral gap γ determines convergence rate:
/// transient cost ≤ e^{-γt}.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicalRealization {
    pub label: String,
    pub spectral_gap: f64,
    /// The generator L — the "physics" driving relaxation.
    pub generator_description: String,
}

impl DynamicalRealization {
    pub fn new(label: &str, spectral_gap: f64, generator_description: &str) -> Self {
        Self {
            label: label.to_string(),
            spectral_gap,
            generator_description: generator_description.to_string(),
        }
    }

    /// Compute transient cost bound at time t: e^{-γt}
    pub fn transient_cost_bound(&self, t: f64) -> f64 {
        (-self.spectral_gap * t).exp()
    }

    /// Time to reach a given tolerance ε from the solution.
    pub fn time_to_tolerance(&self, epsilon: f64) -> f64 {
        if epsilon <= 0.0 || self.spectral_gap <= 0.0 {
            f64::INFINITY
        } else {
            -epsilon.ln() / self.spectral_gap
        }
    }
}

/// Zero-Cost Theorem check.
///
/// If the state is born inside S (via the inclusion i), then φ_t(x) = x for all t.
/// The computation costs zero because there's nothing to compute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroCostTheorem {
    pub is_inside_solution: bool,
    pub cost: f64, // 0.0 if inside, otherwise the residue
}

impl ZeroCostTheorem {
    /// Check zero-cost condition for a state born inside the solution space.
    pub fn check(is_in_solution: bool, dynamical: &DynamicalRealization, t: f64) -> Self {
        if is_in_solution {
            Self {
                is_inside_solution: true,
                cost: 0.0,
            }
        } else {
            Self {
                is_inside_solution: false,
                cost: dynamical.transient_cost_bound(t),
            }
        }
    }
}

/// Naturality Boundary: the irreducible runtime residue.
///
/// Residue = K(answer | structure) — the Kolmogorov complexity of the answer
/// given the structure. Natural computations have zero residue. Non-natural
/// computations have positive residue — this is the irreducible cost.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalityBoundary {
    /// Kolmogorov complexity of the answer given the structure.
    pub residue_bits: f64,
    /// Whether the computation is natural (residue = 0).
    pub is_natural: bool,
    /// Human-readable description of the residue source.
    pub residue_description: String,
}

impl NaturalityBoundary {
    pub fn new(residue_bits: f64, description: &str) -> Self {
        Self {
            residue_bits,
            is_natural: residue_bits == 0.0,
            residue_description: description.to_string(),
        }
    }

    /// A fully natural computation — structure completely determines the answer.
    pub fn natural() -> Self {
        Self::new(0.0, "Structure fully determines answer")
    }

    /// Compute naturality ratio: how much of the computation is eliminable.
    /// Returns 1.0 for fully natural, approaches 0 for fully non-natural.
    pub fn eliminability(&self, total_complexity_bits: f64) -> f64 {
        if total_complexity_bits <= 0.0 {
            1.0
        } else {
            1.0 - (self.residue_bits / total_complexity_bits).min(1.0)
        }
    }
}

/// A complete constitutive computation analysis of a system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutiveAnalysis {
    pub system_label: String,
    pub idempotent_description: String,
    pub solution_space_description: String,
    pub spectral_gap: f64,
    pub naturality_boundary: NaturalityBoundary,
    pub zero_cost_applicable: bool,
    pub category: ComputationCategory,
}

/// The four categories of constitutive computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComputationCategory {
    Hodge,
    CRDT,
    Symplectic,
    Warp,
}

impl std::fmt::Display for ComputationCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComputationCategory::Hodge => write!(f, "Hodge"),
            ComputationCategory::CRDT => write!(f, "CRDT"),
            ComputationCategory::Symplectic => write!(f, "Symplectic"),
            ComputationCategory::Warp => write!(f, "Warp"),
        }
    }
}

/// Conservation-by-construction: the type system IS the proof.
///
/// When constructors factor through the inclusion i: S ↪ A, the type system
/// guarantees zero-cost. You cannot construct a non-solution state through
/// the public API.
#[derive(Debug, Clone)]
pub struct ConservationByConstruction<A, S> {
    /// The inclusion map: S → A. All public constructors go through this.
    pub inclusion: fn(&S) -> A,
    /// The retraction: A → S (partial — only defined on the image of inclusion).
    pub retraction: fn(&A) -> Option<S>,
}

impl<A: Clone + Debug + PartialEq, S: Clone + Debug + PartialEq> ConservationByConstruction<A, S> {
    /// Construct a value in A that is guaranteed to be in S.
    /// This is the ONLY way to create values — it goes through the inclusion.
    pub fn construct(&self, s: &S) -> A {
        (self.inclusion)(s)
    }

    /// Check if a value of A is in the image of S (i.e., is already solved).
    pub fn is_conserved(&self, a: &A) -> bool {
        (self.retraction)(a).is_some()
    }

    /// Verify conservation: every value constructed through inclusion is conserved.
    pub fn verify_conservation(&self, test_values: &[S]) -> bool {
        test_values.iter().all(|s| {
            let a = self.construct(s);
            self.is_conserved(&a)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idempotent_concept() {
        // For f64, the idempotent e(x) = x.floor() is idempotent: floor(floor(x)) = floor(x)
        let e = |x: f64| x.floor();
        assert_eq!(e(e(3.7)), e(3.7));
        assert_eq!(e(e(0.5)), e(0.5));
        assert_eq!(e(e(-1.2)), e(-1.2));
    }

    #[test]
    fn test_karoubi_splitting_integer_floor() {
        // A = f64, S = i64, e = floor, r = floor as i64, i = as f64
        let e = |a: &f64| a.floor();
        let r = |a: &f64| *a as i64;
        let i = |s: &i64| *s as f64;

        let solution_points: Vec<i64> = vec![0, 1, -1, 42, -100];
        let ambient_points: Vec<f64> = vec![0.3, 1.7, -0.5, 42.9];

        let splitting = KaroubiSplitting::<f64, i64>::new("f64", "i64", 1.0);
        let verification = splitting.verify_splitting(&r, &i, &e, &solution_points, &ambient_points);
        assert!(verification.ri_is_id, "r∘i = id_S");
        assert!(verification.ir_is_e, "i∘r = e");
        assert!(verification.is_valid);
    }

    #[test]
    fn test_dynamical_realization_transient_cost() {
        let dyn_real = DynamicalRealization::new("test", 2.0, "generator");
        assert!(dyn_real.transient_cost_bound(0.0) - 1.0 < 1e-10);
        assert!(dyn_real.transient_cost_bound(1.0) < 0.2);
        assert!(dyn_real.transient_cost_bound(5.0) < 1e-4);
    }

    #[test]
    fn test_dynamical_time_to_tolerance() {
        let dyn_real = DynamicalRealization::new("test", 1.0, "generator");
        let t = dyn_real.time_to_tolerance(0.01);
        assert!(t > 4.0 && t < 5.0, "time to 0.01 tolerance should be ~4.6, got {}", t);
    }

    #[test]
    fn test_zero_cost_inside_solution() {
        let dyn_real = DynamicalRealization::new("test", 1.0, "gen");
        let result = ZeroCostTheorem::check(true, &dyn_real, 1.0);
        assert!(result.is_inside_solution);
        assert_eq!(result.cost, 0.0);
    }

    #[test]
    fn test_zero_cost_outside_solution() {
        let dyn_real = DynamicalRealization::new("test", 1.0, "gen");
        let result = ZeroCostTheorem::check(false, &dyn_real, 1.0);
        assert!(!result.is_inside_solution);
        assert!(result.cost > 0.0);
    }

    #[test]
    fn test_naturality_boundary_natural() {
        let nb = NaturalityBoundary::natural();
        assert!(nb.is_natural);
        assert_eq!(nb.residue_bits, 0.0);
    }

    #[test]
    fn test_naturality_boundary_non_natural() {
        let nb = NaturalityBoundary::new(42.0, "some residue");
        assert!(!nb.is_natural);
        assert_eq!(nb.residue_bits, 42.0);
    }

    #[test]
    fn test_eliminability() {
        let nb = NaturalityBoundary::new(25.0, "residue");
        assert!((nb.eliminability(100.0) - 0.75).abs() < 1e-10);
    }

    #[test]
    fn test_conservation_by_construction() {
        let cbc: ConservationByConstruction<f64, i64> = ConservationByConstruction {
            inclusion: |s: &i64| *s as f64,
            retraction: |a: &f64| {
                let rounded = a.round();
                if (*a - rounded).abs() < 1e-10 {
                    Some(rounded as i64)
                } else {
                    None
                }
            },
        };
        assert!(cbc.is_conserved(&3.0));
        assert!(!cbc.is_conserved(&3.5));
        assert!(cbc.verify_conservation(&[1, 2, 3, -5]));
    }

    #[test]
    fn test_spectral_gap_is_the_design_knob() {
        // Larger gap → faster convergence
        let fast = DynamicalRealization::new("fast", 10.0, "gen");
        let slow = DynamicalRealization::new("slow", 0.1, "gen");
        assert!(fast.transient_cost_bound(1.0) < slow.transient_cost_bound(1.0));
    }

    #[test]
    fn test_karoubi_splitting_identity() {
        // The trivial splitting: A = S, e = id
        let e = |a: &f64| *a;
        let r = |a: &f64| *a;
        let i = |s: &f64| *s;
        let splitting = KaroubiSplitting::<f64, f64>::new("f64", "f64", f64::INFINITY);
        let v = splitting.verify_splitting(&r, &i, &e, &[1.0, 2.0], &[3.0, 4.0]);
        assert!(v.is_valid);
    }
}
