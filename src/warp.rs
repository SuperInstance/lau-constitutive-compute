//! Warp realization: e = ballot→broadcast, S = agreed predicate, γ = hardware latency.
//!
//! In distributed consensus (Warp/Paxos/Raft), the idempotent is the commit process:
//! ballot proposes → broadcast to quorum → agreed state.
//! The solution space S is the set of globally-agreed predicates.
//! The spectral gap γ is the hardware latency bound.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A ballot number for consensus.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ballot(pub u64, pub String);

/// A warp message in the consensus protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WarpMessage<T: Clone + PartialEq> {
    /// Phase 1: Prepare(b) — propose ballot b.
    Prepare { ballot: Ballot },
    /// Phase 1b: Promise(b, accepted) — promise not to accept < b, report accepted.
    Promise { ballot: Ballot, accepted: Option<(Ballot, T)> },
    /// Phase 2a: Accept(b, value) — propose value at ballot b.
    Accept { ballot: Ballot, value: T },
    /// Phase 2b: Accepted(b, value) — accept value at ballot b.
    Accepted { ballot: Ballot, value: T },
    /// Commit — value is decided.
    Decided { value: T },
}

/// The state of a single acceptor node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptorState<T: Clone + PartialEq> {
    pub node_id: String,
    pub promised_ballot: Option<Ballot>,
    pub accepted_ballot: Option<Ballot>,
    pub accepted_value: Option<T>,
    pub decided_value: Option<T>,
}

impl<T: Clone + PartialEq> AcceptorState<T> {
    pub fn new(node_id: &str) -> Self {
        Self {
            node_id: node_id.to_string(),
            promised_ballot: None,
            accepted_ballot: None,
            accepted_value: None,
            decided_value: None,
        }
    }

    /// Handle a Prepare message (Phase 1a).
    pub fn handle_prepare(&mut self, ballot: &Ballot) -> Option<WarpMessage<T>> {
        match &self.promised_ballot {
            Some(promised) if ballot <= promised => None, // Reject
            _ => {
                self.promised_ballot = Some(ballot.clone());
                Some(WarpMessage::Promise {
                    ballot: ballot.clone(),
                    accepted: self.accepted_ballot.as_ref()
                        .and_then(|ab| self.accepted_value.as_ref().map(|v| (ab.clone(), v.clone()))),
                })
            }
        }
    }

    /// Handle an Accept message (Phase 2a).
    pub fn handle_accept(&mut self, ballot: &Ballot, value: &T) -> Option<WarpMessage<T>> {
        match &self.promised_ballot {
            Some(promised) if ballot < promised => None, // Reject
            _ => {
                self.promised_ballot = Some(ballot.clone());
                self.accepted_ballot = Some(ballot.clone());
                self.accepted_value = Some(value.clone());
                Some(WarpMessage::Accepted {
                    ballot: ballot.clone(),
                    value: value.clone(),
                })
            }
        }
    }

    /// Decide a value.
    pub fn decide(&mut self, value: &T) {
        self.decided_value = Some(value.clone());
    }

    /// Is this node in the agreed state?
    pub fn is_decided(&self) -> bool {
        self.decided_value.is_some()
    }
}

/// Warp Realization of constitutive computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarpRealization<T: Clone + PartialEq> {
    pub acceptors: Vec<AcceptorState<T>>,
    pub quorum_size: usize,
    /// Hardware latency bound (spectral gap analog).
    pub latency_bound_ms: f64,
}

impl<T: Clone + PartialEq + Debug> WarpRealization<T> {
    pub fn new(node_ids: &[&str], latency_bound_ms: f64) -> Self {
        let acceptors = node_ids.iter()
            .map(|id| AcceptorState::new(id))
            .collect();
        let quorum_size = node_ids.len() / 2 + 1;
        Self {
            acceptors,
            quorum_size,
            latency_bound_ms,
        }
    }

    /// Run a full consensus round: propose value via ballot.
    pub fn run_consensus(&mut self, ballot: Ballot, value: T) -> WarpConsensusResult<T> {
        // Phase 1: Prepare
        let mut promises = 0;
        let mut highest_accepted: Option<(Ballot, T)> = None;

        for acceptor in &mut self.acceptors {
            if let Some(WarpMessage::Promise { accepted, .. }) = acceptor.handle_prepare(&ballot) {
                promises += 1;
                if let Some((b, v)) = accepted {
                    match &highest_accepted {
                        None => highest_accepted = Some((b, v)),
                        Some((hb, _)) if &b > hb => highest_accepted = Some((b, v)),
                        _ => {}
                    }
                }
            }
        }

        if promises < self.quorum_size {
            return WarpConsensusResult::Failed("No quorum for prepare".into());
        }

        // Use the highest previously accepted value, or our proposed value
        let value_to_propose = highest_accepted.map(|(_, v)| v).unwrap_or(value);

        // Phase 2: Accept
        let mut accepts = 0;
        for acceptor in &mut self.acceptors {
            if let Some(WarpMessage::Accepted { .. }) = acceptor.handle_accept(&ballot, &value_to_propose) {
                accepts += 1;
            }
        }

        if accepts < self.quorum_size {
            return WarpConsensusResult::Failed("No quorum for accept".into());
        }

        // Decide
        let decided = value_to_propose.clone();
        for acceptor in &mut self.acceptors {
            acceptor.decide(&value_to_propose);
        }

        WarpConsensusResult::Decided(decided)
    }

    /// The idempotent: re-proposing already-decided value returns same value.
    pub fn idempotent_check(&self, decided_value: &T) -> bool {
        self.acceptors.iter().all(|a| {
            a.decided_value.as_ref() == Some(decided_value)
        })
    }

    /// Zero-cost: if all nodes already decided, no messages needed.
    pub fn zero_cost_check(&self) -> bool {
        let all_decided = self.acceptors.iter().all(|a| a.is_decided());
        if all_decided {
            // All decided same value?
            let first = &self.acceptors[0].decided_value;
            self.acceptors.iter().all(|a| a.decided_value == *first)
        } else {
            false
        }
    }

    /// Compute the agreed predicate (solution space S).
    pub fn agreed_value(&self) -> Option<T> {
        if self.zero_cost_check() {
            self.acceptors[0].decided_value.clone()
        } else {
            None
        }
    }

    /// The spectral gap: inverse of latency bound.
    pub fn spectral_gap(&self) -> f64 {
        1.0 / self.latency_bound_ms.max(1.0)
    }

    /// Rounds to convergence given network reliability.
    pub fn rounds_to_converge(&self, reliability: f64) -> f64 {
        // Expected rounds = 1 / (reliability^quorum)
        let quorum_prob = reliability.powi(self.quorum_size as i32);
        1.0 / quorum_prob
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WarpConsensusResult<T: Clone + PartialEq> {
    Decided(T),
    Failed(String),
}

use std::fmt::Debug;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_consensus() {
        let mut warp: WarpRealization<String> = WarpRealization::new(
            &["n1", "n2", "n3"],
            10.0,
        );
        let ballot = Ballot(1, "leader".into());
        let result = warp.run_consensus(ballot, "hello".into());
        assert_eq!(result, WarpConsensusResult::Decided("hello".to_string()));
    }

    #[test]
    fn test_consensus_all_decided() {
        let mut warp: WarpRealization<String> = WarpRealization::new(
            &["n1", "n2", "n3"],
            10.0,
        );
        warp.run_consensus(Ballot(1, "l".into()), "value".into());
        assert!(warp.zero_cost_check());
    }

    #[test]
    fn test_consensus_agreed_value() {
        let mut warp: WarpRealization<i32> = WarpRealization::new(
            &["n1", "n2", "n3"],
            10.0,
        );
        warp.run_consensus(Ballot(1, "l".into()), 42);
        assert_eq!(warp.agreed_value(), Some(42));
    }

    #[test]
    fn test_idempotent_re_proposal() {
        let mut warp: WarpRealization<String> = WarpRealization::new(
            &["n1", "n2", "n3"],
            10.0,
        );
        warp.run_consensus(Ballot(1, "l".into()), "v1".into());
        assert!(warp.idempotent_check(&"v1".to_string()));
    }

    #[test]
    fn test_higher_ballot_overrides() {
        let mut warp: WarpRealization<String> = WarpRealization::new(
            &["n1", "n2", "n3"],
            10.0,
        );
        warp.run_consensus(Ballot(1, "l1".into()), "v1".into());
        let result = warp.run_consensus(Ballot(2, "l2".into()), "v2".into());
        // Since v1 was already accepted, Paxos should preserve it
        assert_eq!(result, WarpConsensusResult::Decided("v1".to_string()));
    }

    #[test]
    fn test_not_decided_initially() {
        let warp: WarpRealization<String> = WarpRealization::new(
            &["n1", "n2", "n3"],
            10.0,
        );
        assert!(!warp.zero_cost_check());
        assert_eq!(warp.agreed_value(), None);
    }

    #[test]
    fn test_spectral_gap() {
        let warp: WarpRealization<String> = WarpRealization::new(
            &["n1", "n2", "n3"],
            10.0,
        );
        assert!(warp.spectral_gap() > 0.0);
    }

    #[test]
    fn test_acceptor_rejects_old_ballot() {
        let mut acceptor: AcceptorState<String> = AcceptorState::new("a1");
        let high = Ballot(5, "l".into());
        let low = Ballot(1, "l".into());

        acceptor.handle_prepare(&high);
        let result = acceptor.handle_prepare(&low);
        assert!(result.is_none(), "Should reject lower ballot");
    }

    #[test]
    fn test_acceptor_accepts_new_ballot() {
        let mut acceptor: AcceptorState<String> = AcceptorState::new("a1");
        let low = Ballot(1, "l".into());
        let high = Ballot(5, "l".into());

        acceptor.handle_prepare(&low);
        let result = acceptor.handle_prepare(&high);
        assert!(result.is_some(), "Should accept higher ballot");
    }

    #[test]
    fn test_five_node_consensus() {
        let mut warp: WarpRealization<i32> = WarpRealization::new(
            &["n1", "n2", "n3", "n4", "n5"],
            5.0,
        );
        let result = warp.run_consensus(Ballot(1, "l".into()), 99);
        assert_eq!(result, WarpConsensusResult::Decided(99));
        assert_eq!(warp.quorum_size, 3);
    }

    #[test]
    fn test_rounds_to_converge() {
        let warp: WarpRealization<String> = WarpRealization::new(
            &["n1", "n2", "n3"],
            10.0,
        );
        let rounds = warp.rounds_to_converge(0.9);
        assert!(rounds > 0.0);
        assert!(rounds < 100.0);
    }
}
