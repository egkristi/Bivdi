//! Bivdi runtime node — the six primitives composed into one running system.
//!
//! Phase 0/1, single-process on Linux. This is the "running implementation":
//! the object store, capability runtime, state engine, event bus, identity
//! service, and agent host wired together into a coherent `Node`, with
//! authority-relevant events flowing onto the event fabric.
//!
//! # Provisional
//!
//! The kernel half of Phase 1 (seL4 VM bring-up) remains out of scope here and
//! depends on the undecided kernel choice (`P-001`, RFC 0004 proposed).

use bivdi_agent::{Agent, AgentHost, Quota};
use bivdi_cap::{Capability, Resource, Right};
use bivdi_event::{Event, EventBus};
use bivdi_identity::IdentityService;
use bivdi_object::Store;
use bivdi_state::{DesiredState, StateEngine};
use std::collections::BTreeMap;
use std::time::Duration;

/// Errors from runtime-node operations.
#[derive(Debug)]
pub enum NodeError {
    NoSuchAgent,
    Denied,
}

/// A composed Bivdi runtime node. Owns all six services and the agents running
/// on top of them, and provides owner-facing operations that publish
/// authority-relevant events onto the event fabric.
///
/// The capability runtime lives inside [`AgentHost`], which is the single
/// source of truth for capability state; the node does not keep a second one.
#[derive(Debug)]
pub struct Node {
    pub store: Store,
    pub state: StateEngine,
    pub events: EventBus,
    pub identities: IdentityService,
    host: AgentHost,
    agents: BTreeMap<u64, Agent>,
}

impl Node {
    pub fn new(initial: DesiredState) -> Self {
        Self {
            store: Store::new(),
            state: StateEngine::new(initial),
            events: EventBus::new(),
            identities: IdentityService::new(),
            host: AgentHost::new(),
            agents: BTreeMap::new(),
        }
    }

    /// Mint a root capability over a resource (owner-only operation), and
    /// record the grant on the event fabric.
    pub fn mint_root(&mut self, resource: Resource, right: Right) -> Capability {
        let cap = self.host.mint_root(resource, right);
        self.events.publish(Event::CapabilityGranted {
            correlation: self.events.new_correlation().0,
            cap: resource.0,
        });
        cap
    }

    /// Spawn an agent attenuated from `source`, and record the grant. Returns
    /// the new agent's id.
    pub fn spawn_agent(
        &mut self,
        source: &Capability,
        right: Right,
        ttl: Duration,
        quota: Quota,
    ) -> Result<u64, NodeError> {
        let agent = self
            .host
            .spawn(source, right, ttl, quota)
            .map_err(|_| NodeError::Denied)?;
        let id = agent.id;
        self.agents.insert(id, agent);
        self.events.publish(Event::CapabilityGranted {
            correlation: self.events.new_correlation().0,
            cap: id,
        });
        Ok(id)
    }

    /// Execute one action on behalf of an agent, recording the outcome on the
    /// event fabric. Returns an error if the action is not authorized, the
    /// lease is expired, or the quota is exhausted.
    pub fn agent_act(
        &mut self,
        agent_id: u64,
        resource: Resource,
        right: Right,
    ) -> Result<(), NodeError> {
        let agent = self
            .agents
            .get_mut(&agent_id)
            .ok_or(NodeError::NoSuchAgent)?;
        let result = self.host.execute(agent, resource, right);
        match result {
            Ok(()) => {
                self.events.publish(Event::CapabilityUsed {
                    correlation: self.events.new_correlation().0,
                    cap: agent_id,
                });
                Ok(())
            }
            Err(_) => {
                self.events.publish(Event::CapabilityRevoked {
                    correlation: self.events.new_correlation().0,
                    cap: agent_id,
                });
                Err(NodeError::Denied)
            }
        }
    }

    /// Reconcile the state engine and return the ordered actions.
    pub fn reconcile(&self) -> Vec<bivdi_state::Action> {
        self.state.reconcile()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn initial() -> DesiredState {
        DesiredState {
            workloads: BTreeMap::new(),
        }
    }

    #[test]
    fn node_composes_and_mints_authority() {
        let mut node = Node::new(initial());
        let res = Resource(1);
        let root = node.mint_root(res, Right::Grant);
        // The minted capability is usable to spawn (i.e., the host sees it).
        let agent_id = node
            .spawn_agent(
                &root,
                Right::Read,
                Duration::from_secs(60),
                Quota::new(10, 0),
            )
            .unwrap();
        assert!(node.agent_act(agent_id, res, Right::Read).is_ok());
    }

    #[test]
    fn agent_denied_beyond_its_authority() {
        let mut node = Node::new(initial());
        let res = Resource(1);
        let root = node.mint_root(res, Right::Grant);
        let agent_id = node
            .spawn_agent(
                &root,
                Right::Read,
                Duration::from_secs(60),
                Quota::new(10, 0),
            )
            .unwrap();
        assert!(node.agent_act(agent_id, res, Right::Read).is_ok());
        assert!(node.agent_act(agent_id, res, Right::Write).is_err());
    }

    #[test]
    fn authority_events_flow_onto_the_fabric() {
        let mut node = Node::new(initial());
        let res = Resource(1);
        let root = node.mint_root(res, Right::Grant);
        let agent_id = node
            .spawn_agent(
                &root,
                Right::Read,
                Duration::from_secs(60),
                Quota::new(10, 0),
            )
            .unwrap();
        let before = node.events.history().len();
        let _ = node.agent_act(agent_id, res, Right::Read);
        assert!(node.events.history().len() > before);
    }
}
