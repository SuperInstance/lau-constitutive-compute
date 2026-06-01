//! Agent design through constitutive computation.
//!
//! Skills ARE the inclusion i: S ↪ A. Optimal behavior = fixed point of contraction T.
//! Conservation-by-construction means the type system IS the proof.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::core::{ComputationCategory, ConstitutiveAnalysis, DynamicalRealization, NaturalityBoundary};

/// A skill that an agent possesses. Skills ARE inclusions i: S ↪ A.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    /// The solution space this skill provides access to.
    pub solution_space: String,
    /// The spectral gap of this skill (how fast it converges).
    pub spectral_gap: f64,
    /// Whether this skill provides zero-cost computation.
    pub is_zero_cost: bool,
    /// Description of what this skill does.
    pub description: String,
}

impl Skill {
    pub fn new(name: &str, solution_space: &str, spectral_gap: f64, description: &str) -> Self {
        Self {
            name: name.to_string(),
            solution_space: solution_space.to_string(),
            spectral_gap,
            is_zero_cost: spectral_gap == f64::INFINITY,
            description: description.to_string(),
        }
    }
}

/// An agent designed through constitutive computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDesign {
    pub name: String,
    pub skills: BTreeMap<String, Skill>,
    /// The contraction map T: A → A. Optimal behavior = fixed point of T.
    pub contraction_rate: f64,
}

impl AgentDesign {
    pub fn new(name: &str, contraction_rate: f64) -> Self {
        Self {
            name: name.to_string(),
            skills: BTreeMap::new(),
            contraction_rate,
        }
    }

    /// Add a skill (inclusion i: S ↪ A).
    pub fn add_skill(&mut self, skill: Skill) {
        self.skills.insert(skill.name.clone(), skill);
    }

    /// Compute optimal behavior as fixed point of contraction T.
    /// T(x) = x after applying all skills. Fixed point = fully processed state.
    pub fn compute_fixed_point(&self, initial_state: &AgentState) -> AgentState {
        let mut state = initial_state.clone();
        let tolerance = 1e-10;
        let max_iterations = 1000;

        for _ in 0..max_iterations {
            let next = self.apply_contraction(&state);
            let distance = state.distance(&next);
            state = next;
            if distance < tolerance {
                break;
            }
        }
        state
    }

    /// Apply the contraction map T once.
    fn apply_contraction(&self, state: &AgentState) -> AgentState {
        let mut new_state = state.clone();
        // Apply each skill (each is an inclusion that projects toward solution)
        for skill in self.skills.values() {
            if new_state.knowledge.contains_key(&skill.solution_space) {
                // Already in solution space — zero cost
                *new_state.costs.entry(skill.name.clone()).or_insert(0.0) = 0.0;
            } else {
                // Need to converge — cost = e^{-γt}
                let t = new_state.iteration as f64;
                let cost = (-skill.spectral_gap * t).exp();
                *new_state.costs.entry(skill.name.clone()).or_insert(cost) = cost;
                new_state.knowledge.insert(skill.solution_space.clone(), true);
            }
        }
        new_state.iteration += 1;
        new_state
    }

    /// Analyze this agent as a constitutive computation.
    pub fn analyze(&self) -> AgentAnalysis {
        let total_skills = self.skills.len();
        let zero_cost_skills = self.skills.values().filter(|s| s.is_zero_cost).count();
        let avg_gap = if total_skills > 0 {
            self.skills.values().map(|s| s.spectral_gap).sum::<f64>() / total_skills as f64
        } else {
            0.0
        };

        let naturality = if total_skills > 0 {
            zero_cost_skills as f64 / total_skills as f64
        } else {
            0.0
        };

        AgentAnalysis {
            agent_name: self.name.clone(),
            total_skills,
            zero_cost_skills,
            average_spectral_gap: avg_gap,
            contraction_rate: self.contraction_rate,
            naturality_ratio: naturality,
            skills: self.skills.clone(),
        }
    }

    /// Find the idempotent of a skill: the operation e where e∘e = e.
    pub fn find_skill_idempotent(&self, skill_name: &str) -> Option<SkillIdempotent> {
        self.skills.get(skill_name).map(|skill| {
            SkillIdempotent {
                skill_name: skill.name.clone(),
                description: format!("e = {}", skill.description),
                spectral_gap: skill.spectral_gap,
                is_natural: skill.is_zero_cost,
            }
        })
    }
}

/// The state of an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub iteration: u64,
    pub knowledge: BTreeMap<String, bool>,
    pub costs: BTreeMap<String, f64>,
}

impl AgentState {
    pub fn new() -> Self {
        Self {
            iteration: 0,
            knowledge: BTreeMap::new(),
            costs: BTreeMap::new(),
        }
    }

    /// Distance between states (for contraction mapping).
    pub fn distance(&self, other: &Self) -> f64 {
        let mut dist = 0.0;
        for (k, v) in &self.knowledge {
            let other_v = other.knowledge.get(k).unwrap_or(&false);
            if v != other_v {
                dist += 1.0;
            }
        }
        for (k, &c) in &self.costs {
            let other_c = other.costs.get(k).unwrap_or(&0.0);
            dist += (c - other_c).abs();
        }
        dist
    }
}

/// Analysis of an agent as a constitutive computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAnalysis {
    pub agent_name: String,
    pub total_skills: usize,
    pub zero_cost_skills: usize,
    pub average_spectral_gap: f64,
    pub contraction_rate: f64,
    pub naturality_ratio: f64,
    pub skills: BTreeMap<String, Skill>,
}

/// The idempotent associated with a skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillIdempotent {
    pub skill_name: String,
    pub description: String,
    pub spectral_gap: f64,
    pub is_natural: bool,
}

/// Analyze an arbitrary system as a constitutive computation.
/// Find its idempotent, compute its gap, identify the residue.
pub fn analyze_system(
    label: &str,
    idempotent_description: &str,
    solution_space: &str,
    spectral_gap: f64,
    naturality_boundary: NaturalityBoundary,
    zero_cost_applicable: bool,
    category: ComputationCategory,
) -> ConstitutiveAnalysis {
    ConstitutiveAnalysis {
        system_label: label.to_string(),
        idempotent_description: idempotent_description.to_string(),
        solution_space_description: solution_space.to_string(),
        spectral_gap,
        naturality_boundary,
        zero_cost_applicable,
        category,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_with_skills() {
        let mut agent = AgentDesign::new("test-agent", 0.5);
        agent.add_skill(Skill::new("read", "knowledge", f64::INFINITY, "read data"));
        agent.add_skill(Skill::new("compute", "results", 1.0, "compute results"));
        assert_eq!(agent.skills.len(), 2);
    }

    #[test]
    fn test_agent_fixed_point_converges() {
        let mut agent = AgentDesign::new("test", 0.5);
        agent.add_skill(Skill::new("skill1", "domain1", 2.0, "s1"));
        let state = AgentState::new();
        let fixed = agent.compute_fixed_point(&state);
        assert!(fixed.iteration > 0);
    }

    #[test]
    fn test_agent_analysis() {
        let mut agent = AgentDesign::new("test", 0.5);
        agent.add_skill(Skill::new("s1", "d1", 1.0, "skill 1"));
        agent.add_skill(Skill::new("s2", "d2", f64::INFINITY, "skill 2"));
        let analysis = agent.analyze();
        assert_eq!(analysis.total_skills, 2);
        assert_eq!(analysis.zero_cost_skills, 1);
        assert!(analysis.naturality_ratio > 0.0);
    }

    #[test]
    fn test_skill_idempotent() {
        let mut agent = AgentDesign::new("test", 0.5);
        agent.add_skill(Skill::new("lookup", "cache", f64::INFINITY, "cache lookup"));
        let idem = agent.find_skill_idempotent("lookup").unwrap();
        assert!(idem.is_natural);
    }

    #[test]
    fn test_zero_cost_skill() {
        let skill = Skill::new("identity", "self", f64::INFINITY, "identity map");
        assert!(skill.is_zero_cost);
    }

    #[test]
    fn test_agent_state_distance() {
        let s1 = AgentState::new();
        let s2 = AgentState::new();
        assert_eq!(s1.distance(&s2), 0.0);
    }

    #[test]
    fn test_analyze_system() {
        let analysis = analyze_system(
            "test-system",
            "projection",
            "kernel",
            1.5,
            NaturalityBoundary::new(10.0, "some residue"),
            false,
            ComputationCategory::Hodge,
        );
        assert_eq!(analysis.system_label, "test-system");
        assert_eq!(analysis.category, ComputationCategory::Hodge);
        assert!(!analysis.naturality_boundary.is_natural);
    }

    #[test]
    fn test_analyze_natural_system() {
        let analysis = analyze_system(
            "natural-system",
            "identity",
            "everything",
            f64::INFINITY,
            NaturalityBoundary::natural(),
            true,
            ComputationCategory::Warp,
        );
        assert!(analysis.zero_cost_applicable);
        assert!(analysis.naturality_boundary.is_natural);
    }

    #[test]
    fn test_contraction_convergence() {
        let mut agent = AgentDesign::new("test", 2.0);
        agent.add_skill(Skill::new("s1", "d1", 5.0, "fast skill"));
        let state = AgentState::new();
        let fixed = agent.compute_fixed_point(&state);
        // Should converge quickly with high gap
        assert!(fixed.iteration < 100);
    }
}
