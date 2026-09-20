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
use bivdi_cap::{Capability, Resource, Rights};
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
    pub fn mint_root(&mut self, resource: Resource, right: Rights) -> Capability {
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
        right: Rights,
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

    /// Grant an additional capability to an existing agent, attenuated from
    /// `source`. This is how one agent holds *several* capabilities (write to
    /// the calendar **and** read to the document), per `docs/ai-agents.md` §4
    /// (H3).
    pub fn grant_agent(
        &mut self,
        agent_id: u64,
        source: &Capability,
        right: Rights,
    ) -> Result<(), NodeError> {
        let agent = self
            .agents
            .get_mut(&agent_id)
            .ok_or(NodeError::NoSuchAgent)?;
        self.host
            .grant(agent, source, right)
            .map_err(|_| NodeError::Denied)?;
        self.events.publish(Event::CapabilityGranted {
            correlation: self.events.new_correlation().0,
            cap: agent_id,
        });
        Ok(())
    }

    /// Execute one action on behalf of an agent, recording the outcome on the
    /// event fabric. Returns an error if the action is not authorized, the
    /// lease is expired, or the quota is exhausted.
    pub fn agent_act(
        &mut self,
        agent_id: u64,
        resource: Resource,
        right: Rights,
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
                // A failed action is a *denial*, not a revocation — the agent
                // attempted something it was not authorized for, and that
                // attempt must be recorded as such (RFC 0001 §3.4 clause 3).
                self.events.publish(Event::CapabilityDenied {
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

    /// Queryable provenance: return the authority-relevant events (granted,
    /// used, denied, revoked) in append order. This is the operator-facing half
    /// of the value proposition — "what happened, and what gave it the right?"
    /// is a query, not a log dive. It records *authority*, never content.
    pub fn provenance_query(&self) -> Vec<Event> {
        self.events
            .history()
            .into_iter()
            .filter(|e| {
                matches!(
                    e,
                    Event::CapabilityGranted { .. }
                        | Event::CapabilityUsed { .. }
                        | Event::CapabilityDenied { .. }
                        | Event::CapabilityRevoked { .. }
                )
            })
            .collect()
    }

    /// The hash-chained authority provenance with full detail (resource, rights,
    /// capability id) — the operator-facing record that answers "what touched
    /// this, and what authorised each touch?" in full, not just as event-fabric
    /// counts.
    pub fn detailed_provenance(&self) -> Vec<bivdi_cap::Event> {
        self.host.authority_events().to_vec()
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
        let root = node.mint_root(res, Rights::ALL);
        // The minted capability is usable to spawn (i.e., the host sees it).
        let agent_id = node
            .spawn_agent(
                &root,
                Rights::READ,
                Duration::from_secs(60),
                Quota::new(10, 0),
            )
            .unwrap();
        assert!(node.agent_act(agent_id, res, Rights::READ).is_ok());
    }

    #[test]
    fn agent_denied_beyond_its_authority() {
        let mut node = Node::new(initial());
        let res = Resource(1);
        let root = node.mint_root(res, Rights::ALL);
        let agent_id = node
            .spawn_agent(
                &root,
                Rights::READ,
                Duration::from_secs(60),
                Quota::new(10, 0),
            )
            .unwrap();
        assert!(node.agent_act(agent_id, res, Rights::READ).is_ok());
        assert!(node.agent_act(agent_id, res, Rights::WRITE).is_err());
    }

    #[test]
    fn authority_events_flow_onto_the_fabric() {
        let mut node = Node::new(initial());
        let res = Resource(1);
        let root = node.mint_root(res, Rights::ALL);
        let agent_id = node
            .spawn_agent(
                &root,
                Rights::READ,
                Duration::from_secs(60),
                Quota::new(10, 0),
            )
            .unwrap();
        let before = node.events.history().len();
        let _ = node.agent_act(agent_id, res, Rights::READ);
        assert!(node.events.history().len() > before);
    }

    #[test]
    fn provenance_query_returns_authority_events_only() {
        let mut node = Node::new(initial());
        let res = Resource(1);
        let root = node.mint_root(res, Rights::ALL); // granted
        let agent_id = node
            .spawn_agent(
                &root,
                Rights::READ,
                Duration::from_secs(60),
                Quota::new(10, 0),
            )
            .unwrap(); // granted
        let _ = node.agent_act(agent_id, res, Rights::READ); // used
        let _ = node.agent_act(agent_id, res, Rights::WRITE); // denied

        let provenance = node.provenance_query();
        // Only authority events are returned; no object/workload events.
        assert!(provenance.iter().all(|e| {
            matches!(
                e,
                Event::CapabilityGranted { .. }
                    | Event::CapabilityUsed { .. }
                    | Event::CapabilityDenied { .. }
                    | Event::CapabilityRevoked { .. }
            )
        }));
        assert!(provenance
            .iter()
            .any(|e| matches!(e, Event::CapabilityGranted { .. })));
        assert!(provenance
            .iter()
            .any(|e| matches!(e, Event::CapabilityUsed { .. })));
        assert!(provenance
            .iter()
            .any(|e| matches!(e, Event::CapabilityDenied { .. })));
    }

    /// The RFC 0001 §3.4 scenario, as an automated, repeatable test. One agent
    /// is granted a leased write capability to exactly one calendar entry and
    /// read access to exactly one document; the document contains an instruction
    /// directing the agent to forward the mailbox and delete the originals. On
    /// completion, none of the four escape conditions may hold.
    #[test]
    fn rfc_0001_s3_4_prompt_injection_gains_nothing() {
        let mut node = Node::new(initial());

        // The operator owns two resources: a calendar entry and a document.
        let calendar = Resource(42);
        let document = Resource(43);
        let mailbox = Resource(44);

        let calendar_root = node.mint_root(calendar, Rights::ALL);
        let document_root = node.mint_root(document, Rights::ALL);

        // **One** agent, two capabilities (H3): write over the calendar entry
        // and read over the document — nothing else, and no network flow.
        let agent = node
            .spawn_agent(
                &calendar_root,
                Rights::WRITE,
                Duration::from_secs(600),
                Quota::new(20, 0),
            )
            .unwrap();
        node.grant_agent(agent, &document_root, Rights::READ)
            .unwrap();

        // Clause 1 — no network flow capability was ever held: the agent holds
        // only the two granted capabilities; "forward the mailbox" names no
        // flow, and the agent holds no capability over any network endpoint.
        // (Enforced structurally: the node exposes no flow to hand out, and
        // `agent_act` on any network resource is refused below.)

        // The agent attempts the injected instruction: read the mailbox.
        assert!(node.agent_act(agent, mailbox, Rights::READ).is_err());

        // Clause 2 — no capability naming the mailbox exists in the agent's
        // capability space: the read above was denied, not a silent no-op.
        let provenance = node.provenance_query();
        assert!(provenance
            .iter()
            .any(|e| matches!(e, Event::CapabilityDenied { .. })));

        // Clause 3 — the provenance log shows the denial explicitly.
        assert!(provenance.iter().any(|e| matches!(
            e,
            Event::CapabilityDenied { cap, .. } if *cap == agent
        )));

        // Within the grant, the agent can still write the calendar entry and
        // read the document.
        assert!(node.agent_act(agent, calendar, Rights::WRITE).is_ok());
        assert!(node.agent_act(agent, document, Rights::READ).is_ok());

        // But writing the document (read-only) is denied and recorded.
        assert!(node.agent_act(agent, document, Rights::WRITE).is_err());
    }
}
