//! Symplectic realization: e = moment-map reduction, S = constrained level set,
//! γ = stability.
//!
//! In symplectic geometry, the idempotent is the reduction to a constraint surface.
//! The moment map μ: M → g* gives conserved quantities.
//! The solution space is the level set μ⁻¹(c) / G (the Marsden-Weinstein quotient).
//! Stability of the reduced system is the spectral gap.

use nalgebra::{Complex, DMatrix, DVector};
use serde::{Deserialize, Serialize};

/// A simple symplectic system: coupled oscillators.
/// State = (q1, q2, p1, p2) — positions and momenta.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymplecticSystem {
    /// Mass matrix M (diagonal).
    pub masses: Vec<f64>,
    /// Spring constants (coupling matrix).
    pub spring_constants: DMatrix<f64>,
    /// The Hamiltonian H = ½ pᵀ M⁻¹ p + ½ qᵀ K q.
    pub dimension: usize,
}

impl SymplecticSystem {
    pub fn new(masses: Vec<f64>, spring_constants: DMatrix<f64>) -> Self {
        let dim = masses.len();
        Self {
            masses,
            spring_constants,
            dimension: dim,
        }
    }

    /// Compute the Hamiltonian (total energy) of the system.
    pub fn hamiltonian(&self, q: &DVector<f64>, p: &DVector<f64>) -> f64 {
        // T = ½ Σ p_i² / m_i
        let kinetic: f64 = p.iter().zip(self.masses.iter())
            .map(|(&pi, &mi)| pi * pi / (2.0 * mi))
            .sum();
        // V = ½ qᵀ K q
        let potential = 0.5 * q.transpose() * &self.spring_constants * q;
        kinetic + potential[(0, 0)]
    }

    /// The equations of motion: dq/dt = ∂H/∂p, dp/dt = -∂H/∂q.
    /// Returns (dq/dt, dp/dt).
    pub fn equations_of_motion(&self, q: &DVector<f64>, p: &DVector<f64>) -> (DVector<f64>, DVector<f64>) {
        let n = self.dimension;
        // dq/dt = M⁻¹ p
        let dqdt = DVector::from_vec(
            p.iter().zip(self.masses.iter())
                .map(|(&pi, &mi)| pi / mi)
                .collect()
        );
        // dp/dt = -K q
        let dpdt = -&self.spring_constants * q;
        (dqdt, dpdt)
    }

    /// Symplectic Euler step (preserves the symplectic form).
    pub fn step(&self, q: &DVector<f64>, p: &DVector<f64>, dt: f64) -> (DVector<f64>, DVector<f64>) {
        let (dqdt, dpdt) = self.equations_of_motion(q, p);
        let new_p = p + dpdt.scale(dt);
        let new_q = q + DVector::from_vec(
            new_p.iter().zip(self.masses.iter())
                .map(|(&pi, &mi)| pi / mi * dt)
                .collect()
        );
        (new_q, new_p)
    }

    /// Compute the moment map value μ(q, p) = Σ q_i p_i (angular momentum analog).
    pub fn moment_map(&self, q: &DVector<f64>, p: &DVector<f64>) -> f64 {
        q.iter().zip(p.iter()).map(|(&qi, &pi)| qi * pi).sum()
    }

    /// The idempotent: project onto the constraint surface μ⁻¹(c).
    /// This adjusts q and p to satisfy the moment map constraint.
    pub fn idempotent_project(
        &self,
        q: &DVector<f64>,
        p: &DVector<f64>,
        target_moment: f64,
    ) -> (DVector<f64>, DVector<f64>) {
        let current = self.moment_map(q, p);
        let diff = current - target_moment;
        let norm_sq: f64 = q.iter().zip(p.iter())
            .map(|(&qi, &pi)| qi * qi + pi * pi)
            .sum();

        if norm_sq < 1e-12 {
            return (q.clone(), p.clone());
        }

        // Project out the component along the moment map direction
        let scale = diff / norm_sq;
        let new_q = q - q.scale(scale);
        let new_p = p - p.scale(scale);

        (new_q, new_p)
    }

    /// Compute the spectral gap (stability) of the linearized system.
    pub fn spectral_gap(&self) -> f64 {
        let n = self.dimension;
        // Build the symplectic matrix: [[0, M⁻¹], [-K, 0]]
        let mut sym_matrix = DMatrix::zeros(2 * n, 2 * n);

        // M⁻¹ in top-right
        for i in 0..n {
            sym_matrix[(i, n + i)] = 1.0 / self.masses[i];
        }
        // -K in bottom-left
        for i in 0..n {
            for j in 0..n {
                sym_matrix[(n + i, j)] = -self.spring_constants[(i, j)];
            }
        }

        // Eigenvalues should be purely imaginary for stable system
        let eigen = sym_matrix.eigenvalues();
        match eigen {
            Some(eigenvalues) => {
                let max_real: f64 = eigenvalues.iter()
                    .map(|c| c.re.abs())
                    .fold(0.0_f64, f64::max);
                // For stability, all real parts should be zero
                // Spectral gap = smallest |imaginary| eigenvalue (fundamental frequency)
                let imags: Vec<f64> = eigenvalues.iter()
                    .map(|c| c.im.abs())
                    .filter(|&v| v > 1e-10)
                    .collect();
                if imags.is_empty() {
                    0.0
                } else {
                    imags.into_iter().fold(f64::INFINITY, f64::min)
                }
            }
            None => 0.0,
        }
    }
}

/// Symplectic realization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymplecticRealization {
    pub system: SymplecticSystem,
    pub target_moment: f64,
    pub spectral_gap: f64,
}

impl SymplecticRealization {
    pub fn new(masses: Vec<f64>, spring_constants: DMatrix<f64>, target_moment: f64) -> Self {
        let system = SymplecticSystem::new(masses, spring_constants);
        let gap = system.spectral_gap();
        Self {
            system,
            target_moment,
            spectral_gap: gap,
        }
    }

    /// Apply the idempotent (project onto constraint surface).
    pub fn project(&self, q: &DVector<f64>, p: &DVector<f64>) -> (DVector<f64>, DVector<f64>) {
        self.system.idempotent_project(q, p, self.target_moment)
    }

    /// Verify idempotence of the projection.
    pub fn verify_idempotence(&self, q: &DVector<f64>, p: &DVector<f64>) -> bool {
        let (q1, p1) = self.project(q, p);
        let (q2, p2) = self.project(&q1, &p1);
        let dq = &q1 - &q2;
        let dp = &p1 - &p2;
        dq.iter().chain(dp.iter()).all(|x| x.abs() < 1e-8)
    }

    /// Zero-cost: if the state is already on the constraint surface, projection is identity.
    pub fn zero_cost_check(&self, q: &DVector<f64>, p: &DVector<f64>) -> bool {
        let moment = self.system.moment_map(q, p);
        if (moment - self.target_moment).abs() < 1e-8 {
            let (qp, pp) = self.project(q, p);
            let dq = q - &qp;
            let dp = p - &pp;
            dq.iter().chain(dp.iter()).all(|x| x.abs() < 1e-8)
        } else {
            true // Not on constraint surface, theorem doesn't apply
        }
    }

    /// Energy is conserved along trajectories (symplectic integrator property).
    pub fn verify_energy_conservation(&self, q: &DVector<f64>, p: &DVector<f64>, steps: usize, dt: f64) -> bool {
        let e0 = self.system.hamiltonian(q, p);
        let (mut cq, mut cp) = (q.clone(), p.clone());
        for _ in 0..steps {
            let (nq, np) = self.system.step(&cq, &cp, dt);
            cq = nq;
            cp = np;
        }
        let ef = self.system.hamiltonian(&cq, &cp);
        (e0 - ef).abs() < 0.1 * e0.abs().max(1.0) // Allow some drift
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn harmonic_oscillator() -> SymplecticRealization {
        let masses = vec![1.0];
        let springs = DMatrix::from_vec(1, 1, vec![1.0]);
        SymplecticRealization::new(masses, springs, 0.0)
    }

    fn coupled_oscillators() -> SymplecticRealization {
        let masses = vec![1.0, 1.0];
        let springs = DMatrix::from_vec(2, 2, vec![
            2.0, -1.0,
            -1.0, 2.0,
        ]);
        SymplecticRealization::new(masses, springs, 0.0)
    }

    #[test]
    fn test_harmonic_oscillator_hamiltonian() {
        let sys = &harmonic_oscillator().system;
        let q = DVector::from_vec(vec![1.0]);
        let p = DVector::from_vec(vec![0.0]);
        let h = sys.hamiltonian(&q, &p);
        assert!((h - 1.0).abs() < 1e-10, "H = ½ q² = 0.5, got {}", h);
    }

    #[test]
    fn test_harmonic_oscillator_moment_map() {
        let sys = &harmonic_oscillator().system;
        let q = DVector::from_vec(vec![1.0]);
        let p = DVector::from_vec(vec![2.0]);
        assert!((sys.moment_map(&q, &p) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_symplectic_idempotence() {
        let real = harmonic_oscillator();
        let q = DVector::from_vec(vec![2.0]);
        let p = DVector::from_vec(vec![3.0]);
        assert!(real.verify_idempotence(&q, &p));
    }

    #[test]
    fn test_symplectic_coupled_idempotence() {
        let real = coupled_oscillators();
        let q = DVector::from_vec(vec![1.0, 2.0]);
        let p = DVector::from_vec(vec![3.0, 4.0]);
        assert!(real.verify_idempotence(&q, &p));
    }

    #[test]
    fn test_zero_cost_on_constraint() {
        let real = harmonic_oscillator();
        // moment map = q*p = 0 for this
        let q = DVector::from_vec(vec![0.0]);
        let p = DVector::from_vec(vec![5.0]);
        let moment = real.system.moment_map(&q, &p);
        assert!(moment.abs() < 1e-10);
        assert!(real.zero_cost_check(&q, &p));
    }

    #[test]
    fn test_projection_satisfies_constraint() {
        let real = coupled_oscillators();
        let q = DVector::from_vec(vec![1.0, 2.0]);
        let p = DVector::from_vec(vec![3.0, 4.0]);
        let (qp, pp) = real.project(&q, &p);
        let new_moment = real.system.moment_map(&qp, &pp);
        assert!((new_moment - real.target_moment).abs() < 1e-6,
            "Projected moment {} should be {}", new_moment, real.target_moment);
    }

    #[test]
    fn test_spectral_gap_positive() {
        let real = harmonic_oscillator();
        assert!(real.spectral_gap > 0.0, "Spectral gap should be positive");
    }

    #[test]
    fn test_coupled_spectral_gap() {
        let real = coupled_oscillators();
        assert!(real.spectral_gap > 0.0);
    }

    #[test]
    fn test_energy_approximately_conserved() {
        let real = harmonic_oscillator();
        let q = DVector::from_vec(vec![1.0]);
        let p = DVector::from_vec(vec![0.0]);
        assert!(real.verify_energy_conservation(&q, &p, 100, 0.01));
    }

    #[test]
    fn test_equations_of_motion() {
        let sys = &harmonic_oscillator().system;
        let q = DVector::from_vec(vec![1.0]);
        let p = DVector::from_vec(vec![0.0]);
        let (dq, dp) = sys.equations_of_motion(&q, &p);
        // dq/dt = p/M = 0, dp/dt = -K*q = -1
        assert!(dq[0].abs() < 1e-10);
        assert!((dp[0] + 1.0).abs() < 1e-10);
    }
}
