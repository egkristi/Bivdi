//! Bivdi state engine — Phase 0.
//!
//! Implements the decided declarative-state model from `docs/state-engine.md`:
//! - **desired state** is a value,
//! - **reconciliation** computes a diff against observed state,
//! - **generations** are immutable; activation is transactional; rollback is a
//!   pointer change.
//!
//! # Provisional
//!
//! The desired-state schema here is a minimal illustrative JSON value; the real
//! schema belongs to spec v0.1 and the IDL decision.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// A desired-state document (minimal, illustrative). The real schema is part of
/// spec v0.1 and depends on the IDL decision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DesiredState {
    /// Workloads by name → target replica count.
    pub workloads: BTreeMap<String, u64>,
}

/// The observed state that the reconciler compares against.
#[derive(Debug, Clone, PartialEq)]
pub struct ObservedState {
    /// Actual running replicas by workload name.
    pub replicas: BTreeMap<String, u64>,
}

/// A concrete, ordered action the reconciler produced.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    Start { workload: String, count: u64 },
    Stop { workload: String, count: u64 },
}

/// One immutable generation. Each generation is a full snapshot of desired
/// state; a new generation points at the one it replaced.
#[derive(Debug, Clone)]
pub struct Generation {
    pub number: u64,
    pub desired: DesiredState,
    /// The generation this one replaced (enables O(1) rollback).
    pub previous: Option<u64>,
}

/// The state engine: holds generations, reconciles, and rolls back.
#[derive(Debug)]
pub struct StateEngine {
    /// A single lock guards both the generation list and the active pointer,
    /// so there is no lock-ordering hazard between `apply` and `rollback`.
    inner: Arc<Mutex<EngineInner>>,
    observed: Arc<Mutex<ObservedState>>,
}

#[derive(Debug)]
struct EngineInner {
    generations: Vec<Generation>,
    active: u64,
}

impl StateEngine {
    pub fn new(initial: DesiredState) -> Self {
        let g = Generation {
            number: 0,
            desired: initial,
            previous: None,
        };
        Self {
            inner: Arc::new(Mutex::new(EngineInner {
                generations: vec![g],
                active: 0,
            })),
            observed: Arc::new(Mutex::new(ObservedState {
                replicas: BTreeMap::new(),
            })),
        }
    }

    /// Apply a new desired state as a new immutable generation.
    ///
    /// The new generation links to the *currently active* generation, so that
    /// `apply → rollback → apply` rolls back to the correct predecessor.
    pub fn apply(&self, desired: DesiredState) -> u64 {
        let mut inner = self.inner.lock().unwrap();
        let number = inner.generations.len() as u64;
        let previous = Some(inner.active);
        inner.generations.push(Generation {
            number,
            desired,
            previous,
        });
        inner.active = number;
        number
    }

    /// The currently active generation number.
    pub fn active(&self) -> u64 {
        self.inner.lock().unwrap().active
    }

    /// The desired state of the active generation.
    pub fn active_desired(&self) -> DesiredState {
        let inner = self.inner.lock().unwrap();
        inner.generations[inner.active as usize].desired.clone()
    }

    /// Reconcile desired vs. observed, returning the ordered actions to take.
    pub fn reconcile(&self) -> Vec<Action> {
        let desired = self.active_desired();
        let observed = self.observed.lock().unwrap().clone();
        let mut actions = Vec::new();
        for (name, want) in &desired.workloads {
            let have = observed.replicas.get(name).copied().unwrap_or(0);
            match want.cmp(&have) {
                std::cmp::Ordering::Greater => actions.push(Action::Start {
                    workload: name.clone(),
                    count: want - have,
                }),
                std::cmp::Ordering::Less => actions.push(Action::Stop {
                    workload: name.clone(),
                    count: have - want,
                }),
                std::cmp::Ordering::Equal => {}
            }
        }
        // Stop any observed workload that is not desired.
        for (name, have) in &observed.replicas {
            if !desired.workloads.contains_key(name) && *have > 0 {
                actions.push(Action::Stop {
                    workload: name.clone(),
                    count: *have,
                });
            }
        }
        actions
    }

    /// Record the current observed state (what actually runs).
    pub fn observe(&self, replicas: BTreeMap<String, u64>) {
        *self.observed.lock().unwrap() = ObservedState { replicas };
    }

    /// Roll back to a previous generation (transactional: just a pointer move).
    pub fn rollback(&self) -> Option<u64> {
        let mut inner = self.inner.lock().unwrap();
        let prev = inner.generations[inner.active as usize].previous?;
        inner.active = prev;
        Some(prev)
    }

    /// The full generation history (for inspection).
    pub fn history(&self) -> Vec<(u64, DesiredState)> {
        self.inner
            .lock()
            .unwrap()
            .generations
            .iter()
            .map(|g| (g.number, g.desired.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ds(w: &[(&str, u64)]) -> DesiredState {
        DesiredState {
            workloads: w.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        }
    }

    #[test]
    fn reconcile_starts_missing_replicas() {
        let e = StateEngine::new(ds(&[("web", 3)]));
        e.observe(BTreeMap::from([("web".to_string(), 1)]));
        assert_eq!(
            e.reconcile(),
            vec![Action::Start {
                workload: "web".to_string(),
                count: 2
            }]
        );
    }

    #[test]
    fn reconcile_stops_excess_and_removed() {
        let e = StateEngine::new(ds(&[("web", 1)]));
        e.observe(BTreeMap::from([
            ("web".to_string(), 3),
            ("db".to_string(), 2),
        ]));
        let mut actions = e.reconcile();
        actions.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
        assert_eq!(
            actions,
            vec![
                Action::Stop {
                    workload: "db".to_string(),
                    count: 2
                },
                Action::Stop {
                    workload: "web".to_string(),
                    count: 2
                },
            ]
        );
    }

    #[test]
    fn generations_are_immutable_and_rollback_is_pointer() {
        let e = StateEngine::new(ds(&[("web", 1)]));
        let g1 = e.apply(ds(&[("web", 2)]));
        assert_eq!(e.active(), g1);
        assert_eq!(e.active_desired(), ds(&[("web", 2)]));
        let rolled = e.rollback().unwrap();
        assert_eq!(rolled, 0);
        assert_eq!(e.active_desired(), ds(&[("web", 1)]));
    }

    #[test]
    fn rollback_then_apply_links_to_correct_predecessor() {
        let e = StateEngine::new(ds(&[("web", 1)])); // gen 0
        e.apply(ds(&[("web", 2)])); // gen 1
        e.apply(ds(&[("web", 3)])); // gen 2
                                    // Roll back to gen 1.
        assert_eq!(e.rollback().unwrap(), 1);
        // Apply a new generation from gen 1; it must link to gen 1.
        let g3 = e.apply(ds(&[("web", 4)]));
        assert_eq!(e.active(), g3);
        // Rolling back from g3 must return to gen 1 (not gen 0).
        assert_eq!(e.rollback().unwrap(), 1);
        assert_eq!(e.active_desired(), ds(&[("web", 2)]));
    }
}
