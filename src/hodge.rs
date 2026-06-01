//! Hodge realization: e = 1 - ΔG, S = harmonic forms, γ = spectral gap of Laplacian.
//!
//! The Hodge decomposition splits any form into exact + coexact + harmonic.
//! The idempotent e projects onto the harmonic component.
//! The spectral gap of the Laplacian determines convergence rate.

use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};

/// Hodge realization of constitutive computation.
///
/// Works with the discrete Laplacian on a graph/mesh.
/// The idempotent e = I - ΔG where G is the Green's function (pseudoinverse of Δ).
/// The solution space S is the harmonic forms (kernel of Δ).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HodgeRealization {
    /// The Laplacian matrix Δ.
    pub laplacian: DMatrix<f64>,
    /// The pseudoinverse (Green's function) G = Δ⁺.
    pub greens_function: DMatrix<f64>,
    /// The idempotent e = I - ΔG.
    pub idempotent: DMatrix<f64>,
    /// Dimension of the harmonic space (kernel of Δ).
    pub harmonic_dimension: usize,
    /// The spectral gap γ.
    pub spectral_gap: f64,
    /// Eigenvalues of the Laplacian.
    pub eigenvalues: Vec<f64>,
}

impl HodgeRealization {
    /// Build a Hodge realization from a Laplacian matrix.
    pub fn from_laplacian(laplacian: DMatrix<f64>) -> Self {
        let n = laplacian.nrows();
        let eigen = laplacian.clone().symmetric_eigen();
        let eigenvalues: Vec<f64> = eigen.eigenvalues.iter().copied().collect();

        // Spectral gap: smallest non-zero eigenvalue
        let mut sorted = eigenvalues.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let spectral_gap = sorted.iter()
            .find(|&&v| v > 1e-10)
            .copied()
            .unwrap_or(0.0);

        // Harmonic dimension: number of zero eigenvalues
        let harmonic_dim = eigenvalues.iter().filter(|&&v| v.abs() < 1e-10).count();

        // Pseudoinverse: invert non-zero eigenvalues, zero out zero eigenvalues
        let mut pinv_eigenvalues: Vec<f64> = eigenvalues.iter().map(|&v| {
            if v.abs() < 1e-10 { 0.0 } else { 1.0 / v }
        }).collect();

        // G = V * diag(λ⁺) * Vᵀ
        let mut pinv_diag = DMatrix::zeros(n, n);
        for (i, &val) in pinv_eigenvalues.iter().enumerate() {
            pinv_diag[(i, i)] = val;
        }
        let greens_function = &eigen.eigenvectors * pinv_diag * &eigen.eigenvectors.transpose();

        // e = I - ΔG (projects away the harmonic component)
        let identity = DMatrix::identity(n, n);
        let idempotent = &identity - &laplacian * &greens_function;

        HodgeRealization {
            laplacian,
            greens_function,
            idempotent,
            harmonic_dimension: harmonic_dim,
            spectral_gap,
            eigenvalues,
        }
    }

    /// Apply the idempotent to a vector (project onto the non-harmonic component).
    pub fn project(&self, v: &DVector<f64>) -> DVector<f64> {
        &self.idempotent * v
    }

    /// Extract the harmonic component (the kernel projection).
    pub fn harmonic_projection(&self, v: &DVector<f64>) -> DVector<f64> {
        v - &self.project(v)
    }

    /// Verify idempotence: e∘e = e.
    pub fn verify_idempotence(&self) -> bool {
        let ee = &self.idempotent * &self.idempotent;
        let diff = &ee - &self.idempotent;
        diff.iter().all(|x| x.abs() < 1e-8)
    }

    /// Verify e projects onto kernel(Δ).
    pub fn verify_kernel_projection(&self) -> bool {
        let harmonic_basis = self.harmonic_basis();
        for col in harmonic_basis.column_iter() {
            let projected = &self.idempotent * col;
            // For harmonic vectors, e should give zero
            if !projected.iter().all(|x| x.abs() < 1e-6) {
                return false;
            }
        }
        true
    }

    /// Get an orthonormal basis for the harmonic space (kernel of Δ).
    pub fn harmonic_basis(&self) -> DMatrix<f64> {
        let eigen = self.laplacian.clone().symmetric_eigen();
        let n = self.laplacian.nrows();
        let harmonic_cols: Vec<usize> = eigen.eigenvalues.iter()
            .enumerate()
            .filter(|(_, &v)| v.abs() < 1e-10)
            .map(|(i, _)| i)
            .collect();

        let mut basis = DMatrix::zeros(n, harmonic_cols.len().max(1));
        for (j, &col_idx) in harmonic_cols.iter().enumerate() {
            for i in 0..n {
                basis[(i, j)] = eigen.eigenvectors[(i, col_idx)];
            }
        }
        basis
    }

    /// Transient cost at time t: e^{-γt}.
    pub fn transient_cost(&self, t: f64) -> f64 {
        (-self.spectral_gap * t).exp()
    }

    /// Check if a vector is harmonic (in the solution space S).
    pub fn is_harmonic(&self, v: &DVector<f64>) -> bool {
        let lv = &self.laplacian * v;
        lv.iter().all(|x| x.abs() < 1e-8)
    }

    /// Zero-cost theorem: if vector is born harmonic, it stays harmonic.
    pub fn zero_cost_check(&self, v: &DVector<f64>) -> bool {
        if self.is_harmonic(v) {
            // Apply idempotent: should be unchanged (projection kills only non-harmonic)
            let harm = self.harmonic_projection(v);
            (v - &harm).iter().all(|x| x.abs() < 1e-8)
        } else {
            true // Not in S, theorem doesn't claim zero cost
        }
    }
}

/// Build a simple path graph Laplacian of size n.
pub fn path_laplacian(n: usize) -> DMatrix<f64> {
    let mut l = DMatrix::zeros(n, n);
    for i in 0..n {
        if i > 0 {
            l[(i, i)] += 1.0;
            l[(i, i - 1)] -= 1.0;
        }
        if i < n - 1 {
            l[(i, i)] += 1.0;
            l[(i, i + 1)] -= 1.0;
        }
    }
    l
}

/// Build a cycle graph Laplacian of size n.
pub fn cycle_laplacian(n: usize) -> DMatrix<f64> {
    let mut l = DMatrix::zeros(n, n);
    for i in 0..n {
        l[(i, i)] = 2.0;
        l[(i, (i + 1) % n)] -= 1.0;
        l[(i, (i + n - 1) % n)] -= 1.0;
    }
    l
}

/// Build a complete graph Laplacian of size n.
pub fn complete_laplacian(n: usize) -> DMatrix<f64> {
    let mut l = DMatrix::zeros(n, n);
    for i in 0..n {
        l[(i, i)] = (n - 1) as f64;
        for j in 0..n {
            if i != j {
                l[(i, j)] -= 1.0;
            }
        }
    }
    l
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_graph_hodge() {
        let laplacian = path_laplacian(5);
        let hodge = HodgeRealization::from_laplacian(laplacian);
        assert!(hodge.verify_idempotence(), "e∘e = e for path graph");
        assert_eq!(hodge.harmonic_dimension, 0, "Path graph has no harmonic forms");
        assert!(hodge.spectral_gap > 0.0, "Spectral gap should be positive");
    }

    #[test]
    fn test_cycle_graph_hodge() {
        let laplacian = cycle_laplacian(4);
        let hodge = HodgeRealization::from_laplacian(laplacian);
        assert!(hodge.verify_idempotence());
        assert_eq!(hodge.harmonic_dimension, 1, "Cycle graph has 1 harmonic form (constant)");
        assert!(hodge.spectral_gap > 0.0);
    }

    #[test]
    fn test_complete_graph_hodge() {
        let laplacian = complete_laplacian(4);
        let hodge = HodgeRealization::from_laplacian(laplacian);
        assert!(hodge.verify_idempotence());
        assert_eq!(hodge.harmonic_dimension, 1, "Complete graph has 1 harmonic form");
    }

    #[test]
    fn test_harmonic_vector_zero_cost() {
        let laplacian = cycle_laplacian(4);
        let hodge = HodgeRealization::from_laplacian(laplacian);
        let constant = DVector::from_element(4, 1.0);
        assert!(hodge.is_harmonic(&constant));
        assert!(hodge.zero_cost_check(&constant));
    }

    #[test]
    fn test_non_harmonic_vector_decays() {
        let laplacian = path_laplacian(5);
        let hodge = HodgeRealization::from_laplacian(laplacian);
        let v = DVector::from_vec(vec![1.0, 0.0, 0.0, 0.0, 0.0]);
        assert!(!hodge.is_harmonic(&v));
        let projected = hodge.project(&v);
        // After projection, the non-harmonic component should be preserved
        // and the harmonic component (which is zero for path graph) is removed
        assert!(projected.norm() > 0.0);
    }

    #[test]
    fn test_spectral_gap_path_vs_complete() {
        let path = HodgeRealization::from_laplacian(path_laplacian(10));
        let complete = HodgeRealization::from_laplacian(complete_laplacian(10));
        // Complete graph has much larger spectral gap
        assert!(complete.spectral_gap > path.spectral_gap);
    }

    #[test]
    fn test_transient_cost_decreases() {
        let hodge = HodgeRealization::from_laplacian(path_laplacian(5));
        assert!(hodge.transient_cost(1.0) > hodge.transient_cost(2.0));
        assert!(hodge.transient_cost(2.0) > hodge.transient_cost(5.0));
    }

    #[test]
    fn test_kernel_projection() {
        let laplacian = cycle_laplacian(3);
        let hodge = HodgeRealization::from_laplacian(laplacian);
        assert!(hodge.verify_kernel_projection());
    }

    #[test]
    fn test_hodge_projection_idempotent_on_arbitrary_vector() {
        let hodge = HodgeRealization::from_laplacian(path_laplacian(5));
        let v = DVector::from_vec(vec![3.0, -1.0, 4.0, 2.0, -7.0]);
        let p1 = hodge.project(&v);
        let p2 = hodge.project(&p1);
        assert!((p1 - p2).norm() < 1e-8, "Projecting twice should give same result");
    }
}
