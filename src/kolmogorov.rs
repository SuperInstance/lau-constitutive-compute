//! Kolmogorov complexity estimation for the Naturality Boundary.
//!
//! K(answer | structure) — the Kolmogorov complexity of the answer given the structure.
//! This is the irreducible runtime residue. Natural computations have K = 0.
//! Non-natural computations have K > 0.

use serde::{Deserialize, Serialize};

/// A simple estimator for Kolmogorov complexity.
///
/// True K is uncomputable, but we can estimate upper bounds using compression-like measures.
/// The key insight: if structure determines the answer, K(answer|structure) ≈ 0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KolmogorovResidue {
    /// Estimated Kolmogorov complexity of the answer.
    pub k_answer: f64,
    /// Estimated Kolmogorov complexity of the structure.
    pub k_structure: f64,
    /// Estimated conditional Kolmogorov complexity K(answer | structure).
    pub k_conditional: f64,
    /// The naturality ratio: how much structure eliminates computation.
    pub naturality: f64,
    /// Description of the residue source.
    pub description: String,
}

impl KolmogorovResidue {
    /// Estimate Kolmogorov complexity of a string by compression ratio.
    /// We use a simple RLE-inspired estimator.
    pub fn estimate_complexity(data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        // Simple compression: count runs
        let mut compressed_size = 0.0;
        let mut i = 0;
        while i < data.len() {
            let mut run_len = 1;
            while i + run_len < data.len() && data[i + run_len] == data[i] {
                run_len += 1;
            }
            // Encode: value + run length
            compressed_size += 1.0 + (run_len as f64).log2().ceil();
            i += run_len;
        }
        compressed_size
    }

    /// Estimate conditional complexity K(answer | structure) by measuring
    /// how much of the answer is NOT predictable from the structure.
    pub fn estimate_conditional(answer: &[u8], structure: &[u8]) -> f64 {
        let k_answer = Self::estimate_complexity(answer);
        let k_structure = Self::estimate_complexity(structure);

        // Mutual information approximation: shared patterns
        let mutual = Self::estimate_mutual_information(answer, structure);

        // K(answer | structure) ≤ K(answer) - I(answer; structure)
        let k_conditional = (k_answer - mutual).max(0.0);

        k_conditional
    }

    /// Estimate mutual information between two byte sequences.
    fn estimate_mutual_information(a: &[u8], b: &[u8]) -> f64 {
        // Simple approach: count shared byte values
        let a_set: std::collections::HashSet<u8> = a.iter().copied().collect();
        let b_set: std::collections::HashSet<u8> = b.iter().copied().collect();
        let shared = a_set.intersection(&b_set).count() as f64;
        let total_unique = a_set.union(&b_set).count() as f64;
        if total_unique == 0.0 {
            return 0.0;
        }
        // Mutual information ≈ shared_entropy
        let ratio = shared / total_unique;
        ratio * (k_info(total_unique))
    }

    /// Analyze a computation's naturality boundary.
    pub fn analyze(answer: &[u8], structure: &[u8], description: &str) -> Self {
        let k_answer = Self::estimate_complexity(answer);
        let k_structure = Self::estimate_complexity(structure);
        let k_conditional = Self::estimate_conditional(answer, structure);
        let naturality = if k_answer == 0.0 {
            1.0
        } else {
            1.0 - (k_conditional / k_answer).min(1.0)
        };

        KolmogorovResidue {
            k_answer,
            k_structure,
            k_conditional,
            naturality,
            description: description.to_string(),
        }
    }

    /// A fully natural computation (structure determines answer completely).
    pub fn fully_natural(description: &str) -> Self {
        Self {
            k_answer: 0.0,
            k_structure: 0.0,
            k_conditional: 0.0,
            naturality: 1.0,
            description: description.to_string(),
        }
    }
}

/// Shannon entropy of a set of values.
fn k_info(n: f64) -> f64 {
    if n <= 1.0 { 0.0 } else { n.log2() }
}

/// Kolmogorov complexity of common structures.
pub fn structure_complexity(structure_type: &str) -> f64 {
    match structure_type {
        "identity" | "constant" => 1.0,
        "linear" => 10.0,
        "polynomial" => 50.0,
        "exponential" => 30.0,
        "tree" => 100.0,
        "graph" => 200.0,
        "arbitrary" => 1000.0,
        _ => 50.0,
    }
}

/// Compute the naturality boundary for a computation.
/// Returns the residue in bits and the eliminability ratio.
pub fn naturality_boundary(
    answer_description: &str,
    structure_description: &str,
    answer_bytes: &[u8],
    structure_bytes: &[u8],
) -> (f64, f64) {
    let analysis = KolmogorovResidue::analyze(answer_bytes, structure_bytes, "");
    (analysis.k_conditional, analysis.naturality)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_complexity_constant() {
        let data = vec![42u8; 100];
        let k = KolmogorovResidue::estimate_complexity(&data);
        assert!(k < 20.0, "Constant data should have low complexity, got {}", k);
    }

    #[test]
    fn test_estimate_complexity_random() {
        let data: Vec<u8> = (0..100).map(|i| (i * 37 + 13) as u8).collect();
        let k = KolmogorovResidue::estimate_complexity(&data);
        // Should be higher than constant data
        assert!(k > 10.0, "Random-ish data should have higher complexity");
    }

    #[test]
    fn test_natural_computation() {
        let structure = b"identity function";
        let answer = structure; // Answer IS the structure
        let residue = KolmogorovResidue::analyze(answer, structure, "identity");
        assert!(residue.k_conditional < 10.0, "Natural computation should have low residue");
    }

    #[test]
    fn test_fully_natural() {
        let n = KolmogorovResidue::fully_natural("trivial");
        assert_eq!(n.k_conditional, 0.0);
        assert_eq!(n.naturality, 1.0);
    }

    #[test]
    fn test_naturality_ratio_range() {
        let answer = b"the answer to life the universe and everything";
        let structure = b"some structure that partially determines it";
        let analysis = KolmogorovResidue::analyze(answer, structure, "test");
        assert!(analysis.naturality >= 0.0 && analysis.naturality <= 1.0);
    }

    #[test]
    fn test_structure_complexity_known() {
        assert!(structure_complexity("identity") < structure_complexity("graph"));
        assert!(structure_complexity("linear") < structure_complexity("polynomial"));
    }

    #[test]
    fn test_empty_data() {
        let k = KolmogorovResidue::estimate_complexity(&[]);
        assert_eq!(k, 0.0);
    }

    #[test]
    fn test_naturality_boundary_function() {
        let (residue, nat) = naturality_boundary(
            "answer", "structure", b"answer data", b"structure data"
        );
        assert!(residue >= 0.0);
        assert!(nat >= 0.0 && nat <= 1.0);
    }
}
