//! Bivdi agent host — the Runtime's agent-execution core.
//!
//! Implements the decided agent model from `docs/ai-agents.md` on top of the
//! capability runtime:
//! - **attenuated delegation** (an agent only holds what it was handed),
//! - **time-limited** (leases) and **quota-bound** (action/cost budget),
//! - **no escalation** (an agent cannot delegate more than it holds),
//! - **content is data, not instructions** (no ambient grant),
//! - **full provenance** (every action is attributable),
//! - **AI proposes, the OS enforces** (a reviewable plan is enforced by the
//!   host, not by the agent).
//!
//! # Scope
//!
//! This crate is the agent-execution-host half of the product (`D-014`). The
//! microkernel Core is a parked research track (`D-003`); the kernel choice
//! (`P-001`) is deferred indefinitely and does not gate this crate.

use bivdi_cap::{CapRuntime, Capability, Lease, Resource, Rights};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A quota for an agent: maximum actions and maximum delegations.
#[derive(Debug, Clone, Copy)]
pub struct Quota {
    pub max_actions: u64,
    pub max_delegations: u64,
}

impl Quota {
    pub fn new(max_actions: u64, max_delegations: u64) -> Self {
        Self {
            max_actions,
            max_delegations,
        }
    }
}

/// A proposed step in an agent's plan. This is *data*, not instructions: it is
/// enforced by the host and can never grant new authority on its own. The
/// `right` is the authority the step *requests* to exercise; the host checks it
/// against the agent's held capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    /// The resource the step acts on.
    pub resource: u64,
    /// The right the step requires (a rights flag set).
    pub right: Rights,
}

/// A reviewable plan, produced by the AI and enforced by the host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub steps: Vec<Step>,
}

/// An agent, holding a capability set, a lease, and a quota.
///
/// The decided model (`docs/ai-agents.md` §4) gives an agent *several*
/// capabilities — e.g. write to one calendar entry **and** read to one email
/// thread. This struct therefore holds a **set** of capabilities (one per
/// resource), not a single one (H3).
#[derive(Debug, Clone)]
pub struct Agent {
    pub id: u64,
    /// The capabilities this agent holds, each over a distinct resource.
    caps: Vec<Capability>,
    lease: Lease,
    quota: Quota,
    actions_used: u64,
    delegations_used: u64,
}

impl Agent {
    /// The capability this agent holds over `resource`, if any.
    fn cap_for(&self, resource: Resource) -> Option<&Capability> {
        self.caps.iter().find(|c| c.resource() == resource)
    }

    /// The union of rights this agent may exercise across all its capabilities.
    /// Used for the delegation-escalation check: an agent may only delegate a
    /// right it holds over *some* resource.
    fn max_right(&self) -> Rights {
        self.caps
            .iter()
            .fold(Rights::NONE, |acc, c| acc | c.right())
    }
}

/// Errors from agent-host operations.
#[derive(Debug)]
pub enum AgentError {
    /// The agent does not hold the capability it is trying to use.
    NotAuthorized,
    /// The lease has expired.
    Expired,
    /// The quota is exhausted.
    QuotaExceeded,
    /// Delegation would widen authority (forbidden).
    EscalationDenied,
}

/// The agent host. Owns a `CapRuntime` and manages agents against it.
#[derive(Debug)]
pub struct AgentHost {
    runtime: CapRuntime,
    next_agent_id: u64,
}

impl Default for AgentHost {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentHost {
    pub fn new() -> Self {
        Self {
            runtime: CapRuntime::new(),
            next_agent_id: 0,
        }
    }

    /// Mint a new root capability over a resource. This is the **owner
    /// operation** — it is what establishes authority over a resource in the
    /// first place, and it must never be reachable from an agent. Agents only
    /// ever receive capabilities attenuated from something the caller holds.
    pub fn mint_root(&mut self, resource: Resource, right: Rights) -> Capability {
        self.runtime.mint(resource, right)
    }

    /// Spawn an agent from a capability the spawner already holds.
    ///
    /// The agent receives a capability **attenuated from `source`** — it can
    /// never gain authority the spawner did not already hold. `right` must be
    /// a subset of the source's rights; requesting any right the source does
    /// not hold is escalation and is **denied**. No capability is minted here:
    /// minting is reserved for a resource's owner.
    pub fn spawn(
        &mut self,
        source: &Capability,
        right: Rights,
        ttl: Duration,
        quota: Quota,
    ) -> Result<Agent, AgentError> {
        // The agent's lease cannot outlive the source's lease, if any.
        let ttl = if let Some(held) = self.runtime.lease_of(source) {
            ttl.min(held.remaining())
        } else {
            ttl
        };
        // `attenuate` enforces subset inclusion: a right outside the source's
        // set is a widening and is rejected here.
        let capability = self
            .runtime
            .attenuate(source, right)
            .map_err(|_| AgentError::NotAuthorized)?;
        let id = self.next_agent_id;
        self.next_agent_id += 1;
        Ok(Agent {
            id,
            caps: vec![capability],
            lease: Lease::new(ttl),
            quota,
            actions_used: 0,
            delegations_used: 0,
        })
    }

    /// Grant an additional capability to an existing agent, attenuated from a
    /// source the caller holds. This is how one agent comes to hold *several*
    /// capabilities (write to the calendar **and** read to the document),
    /// matching the decided model in `docs/ai-agents.md` §4 (H3).
    pub fn grant(
        &mut self,
        agent: &mut Agent,
        source: &Capability,
        right: Rights,
    ) -> Result<(), AgentError> {
        // Reject escalation: the granted right must be a subset of the source.
        let capability = self
            .runtime
            .attenuate(source, right)
            .map_err(|_| AgentError::NotAuthorized)?;
        // One capability per resource; a second grant over the same resource
        // replaces the first (narrowing, never widening, is fine).
        agent.caps.retain(|c| c.resource() != capability.resource());
        agent.caps.push(capability);
        Ok(())
    }

    /// Execute one plan step on behalf of an agent. This is the enforcement
    /// point: the host checks lease, quota, and authority before acting, and
    /// records the action in provenance. An agent cannot act through authority
    /// it was not handed.
    pub fn execute(
        &mut self,
        agent: &mut Agent,
        resource: Resource,
        required: Rights,
    ) -> Result<(), AgentError> {
        if agent.lease.is_expired() {
            return Err(AgentError::Expired);
        }
        if agent.actions_used >= agent.quota.max_actions {
            return Err(AgentError::QuotaExceeded);
        }
        // Find the capability over this resource; if none, deny — and record
        // the denial explicitly (authority = none), so a "no flow capability"
        // connect attempt is a queryable fact, never a silent drop.
        let cap = match agent.cap_for(resource) {
            Some(cap) => *cap,
            None => {
                self.runtime.record_no_capability(resource, required);
                return Err(AgentError::NotAuthorized);
            }
        };
        if !self.runtime.record_use(&cap, resource, required) {
            return Err(AgentError::NotAuthorized);
        }
        agent.actions_used += 1;
        Ok(())
    }

    /// Delegate a *narrower-or-equal* capability from one agent to a new agent.
    ///
    /// Escalation is denied on all three dimensions:
    /// - the new right cannot exceed the delegator's effective right,
    /// - the new lease is clamped to the delegator's remaining lease,
    /// - the new quota is clamped to the delegator's remaining budget.
    pub fn delegate(
        &mut self,
        from: &mut Agent,
        right: Rights,
        ttl: Duration,
        quota: Quota,
    ) -> Result<Agent, AgentError> {
        if from.lease.is_expired() {
            return Err(AgentError::Expired);
        }
        if from.delegations_used >= from.quota.max_delegations {
            return Err(AgentError::QuotaExceeded);
        }
        if !from.max_right().contains(right) {
            return Err(AgentError::EscalationDenied);
        }
        // Clamp time and quota to what the delegator has left.
        let remaining_actions = from.quota.max_actions.saturating_sub(from.actions_used);
        let remaining_delegations = from
            .quota
            .max_delegations
            .saturating_sub(from.delegations_used);
        let child_ttl = ttl.min(from.lease.remaining());
        let child_quota = Quota::new(
            quota.max_actions.min(remaining_actions),
            quota.max_delegations.min(remaining_delegations),
        );

        from.delegations_used += 1;
        let id = self.next_agent_id;
        self.next_agent_id += 1;
        // The child is delegated the *same* capability set as the parent, but
        // with the rights each capability grants narrowed to `right` where it
        // is a subset — otherwise the capability is dropped.
        let caps: Vec<Capability> = from
            .caps
            .iter()
            .filter_map(|c| {
                let subset = c.right().intersection(right);
                self.runtime.attenuate(c, subset).ok()
            })
            .collect();
        Ok(Agent {
            id,
            caps,
            lease: Lease::new(child_ttl),
            quota: child_quota,
            actions_used: 0,
            delegations_used: 0,
        })
    }

    /// Execute a full plan on behalf of an agent. Each step is enforced in
    /// turn; the first failure aborts the plan. This is "AI proposes, the OS
    /// enforces" — the plan is data, and the host is the enforcement point.
    /// The right each step requires is taken from the step itself and checked
    /// against the agent's held capability (never hard-coded).
    pub fn execute_plan(&mut self, agent: &mut Agent, plan: &Plan) -> Result<(), AgentError> {
        for step in &plan.steps {
            self.execute(agent, Resource(step.resource), step.right)?;
        }
        Ok(())
    }

    /// The underlying provenance log (authority-relevant events).
    pub fn provenance_len(&self) -> usize {
        self.runtime.provenance().len()
    }

    /// The hash-chained authority provenance log, with full detail (resource,
    /// rights, capability ids). This is the operator-facing record: "what
    /// touched this, and what authorised each touch?" answered by the log, not
    /// by a count.
    pub fn authority_events(&self) -> &[bivdi_cap::Event] {
        self.runtime.provenance()
    }

    /// Whether the hash-chained provenance log is intact: `true` iff no entry
    /// has been altered, removed, or reordered. This is the tamper-evident
    /// guarantee the operator can check, not just trust.
    pub fn verify_provenance_chain(&self) -> bool {
        self.runtime.verify_chain()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Convenience: a host with a root capability over a resource.
    fn host_with_root(resource: Resource, right: Rights) -> (AgentHost, Capability) {
        let mut h = AgentHost::new();
        let root = h.mint_root(resource, right);
        (h, root)
    }

    #[test]
    fn spawn_cannot_escalate_beyond_source_right() {
        let res = Resource(10);
        let (mut h, root) = host_with_root(res, Rights::READ);
        // Requesting Write from a Read-only source is escalation: denied, not
        // clamped (subset inclusion, RFC 0002 §5).
        assert!(matches!(
            h.spawn(
                &root,
                Rights::WRITE,
                Duration::from_secs(60),
                Quota::new(10, 0),
            ),
            Err(AgentError::NotAuthorized)
        ));
        // Requesting Read (a subset) is accepted.
        let agent = h
            .spawn(
                &root,
                Rights::READ,
                Duration::from_secs(60),
                Quota::new(10, 0),
            )
            .unwrap();
        assert_eq!(agent.max_right(), Rights::READ);
        // The agent cannot act with Write.
        let mut agent = agent;
        assert!(matches!(
            h.execute(&mut agent, res, Rights::WRITE),
            Err(AgentError::NotAuthorized)
        ));
    }

    #[test]
    fn agent_can_only_act_within_its_authority() {
        let res = Resource(10);
        let (mut h, root) = host_with_root(res, Rights::ALL);
        let mut agent = h
            .spawn(
                &root,
                Rights::READ,
                Duration::from_secs(60),
                Quota::new(10, 0),
            )
            .unwrap();
        assert!(h.execute(&mut agent, res, Rights::READ).is_ok());
        assert!(matches!(
            h.execute(&mut agent, res, Rights::WRITE),
            Err(AgentError::NotAuthorized)
        ));
    }

    #[test]
    fn quota_exhaustion_blocks_further_actions() {
        let res = Resource(10);
        let (mut h, root) = host_with_root(res, Rights::ALL);
        let mut agent = h
            .spawn(
                &root,
                Rights::READ,
                Duration::from_secs(60),
                Quota::new(2, 0),
            )
            .unwrap();
        assert!(h.execute(&mut agent, res, Rights::READ).is_ok());
        assert!(h.execute(&mut agent, res, Rights::READ).is_ok());
        assert!(matches!(
            h.execute(&mut agent, res, Rights::READ),
            Err(AgentError::QuotaExceeded)
        ));
    }

    #[test]
    fn lease_expiry_blocks_action() {
        let res = Resource(10);
        let (mut h, root) = host_with_root(res, Rights::ALL);
        let mut agent = h
            .spawn(
                &root,
                Rights::READ,
                Duration::from_millis(1),
                Quota::new(10, 0),
            )
            .unwrap();
        std::thread::sleep(Duration::from_millis(5));
        assert!(matches!(
            h.execute(&mut agent, res, Rights::READ),
            Err(AgentError::Expired)
        ));
    }

    #[test]
    fn delegation_cannot_escalate_right() {
        let res = Resource(10);
        let (mut h, root) = host_with_root(res, Rights::ALL);
        let mut parent = h
            .spawn(
                &root,
                Rights::WRITE,
                Duration::from_secs(60),
                Quota::new(10, 2),
            )
            .unwrap();
        // Delegating Write (equal) is allowed.
        let child = h
            .delegate(
                &mut parent,
                Rights::WRITE,
                Duration::from_secs(60),
                Quota::new(10, 0),
            )
            .unwrap();
        let _ = child;
        // Delegating Grant (wider) is denied.
        let mut parent2 = h
            .spawn(
                &root,
                Rights::WRITE,
                Duration::from_secs(60),
                Quota::new(10, 2),
            )
            .unwrap();
        assert!(matches!(
            h.delegate(
                &mut parent2,
                Rights::GRANT,
                Duration::from_secs(60),
                Quota::new(10, 0)
            ),
            Err(AgentError::EscalationDenied)
        ));
    }

    #[test]
    fn delegation_clamps_time_and_quota() {
        let res = Resource(10);
        let (mut h, root) = host_with_root(res, Rights::ALL);
        // Parent has a short lease and a 1-action budget.
        let mut parent = h
            .spawn(
                &root,
                Rights::READ,
                Duration::from_millis(30),
                Quota::new(1, 1),
            )
            .unwrap();
        // Child requests a 1-hour lease and a huge quota.
        let mut child = h
            .delegate(
                &mut parent,
                Rights::READ,
                Duration::from_secs(3600),
                Quota::new(1_000_000, 1_000_000),
            )
            .unwrap();
        // The child's quota is clamped to the parent's remaining 1 action.
        assert_eq!(child.quota.max_actions, 1);
        // After the parent's lease (and hence the child's clamped lease) expires,
        // the child cannot act.
        std::thread::sleep(Duration::from_millis(50));
        assert!(matches!(
            h.execute(&mut child, res, Rights::READ),
            Err(AgentError::Expired)
        ));
    }

    #[test]
    fn every_action_is_provenanced() {
        let res = Resource(10);
        let (mut h, root) = host_with_root(res, Rights::ALL);
        let mut agent = h
            .spawn(
                &root,
                Rights::READ,
                Duration::from_secs(60),
                Quota::new(10, 0),
            )
            .unwrap();
        let before = h.provenance_len();
        h.execute(&mut agent, res, Rights::READ).unwrap();
        assert!(h.provenance_len() > before);
    }

    #[test]
    fn plan_is_enforced_by_the_host() {
        let res = Resource(10);
        let (mut h, root) = host_with_root(res, Rights::ALL);
        let mut agent = h
            .spawn(
                &root,
                Rights::READ,
                Duration::from_secs(60),
                Quota::new(10, 0),
            )
            .unwrap();
        // A plan with two read steps executes fully within quota.
        let plan = Plan {
            steps: vec![
                Step {
                    resource: res.0,
                    right: Rights::READ,
                },
                Step {
                    resource: res.0,
                    right: Rights::READ,
                },
            ],
        };
        assert!(h.execute_plan(&mut agent, &plan).is_ok());
        assert_eq!(agent.actions_used, 2);
    }

    #[test]
    fn plan_respects_per_step_right() {
        let res = Resource(10);
        let (mut h, root) = host_with_root(res, Rights::ALL);
        // Agent holds only Read.
        let mut agent = h
            .spawn(
                &root,
                Rights::READ,
                Duration::from_secs(60),
                Quota::new(10, 0),
            )
            .unwrap();
        // A plan whose step requires Write must be denied, even though the
        // agent is otherwise within quota and lease.
        let plan = Plan {
            steps: vec![Step {
                resource: res.0,
                right: Rights::WRITE,
            }],
        };
        assert!(matches!(
            h.execute_plan(&mut agent, &plan),
            Err(AgentError::NotAuthorized)
        ));
    }

    /// One agent can hold *several* capabilities over distinct resources, as
    /// the decided model requires ("write to one calendar entry **and** read to
    /// one email thread"). This is the H3 fix made concrete.
    #[test]
    fn one_agent_holds_multiple_capabilities() {
        let calendar = Resource(42);
        let document = Resource(43);
        let (mut h, cal_root) = host_with_root(calendar, Rights::ALL);
        let doc_root = h.mint_root(document, Rights::ALL);

        // One agent, two grants: write over the calendar, read over the doc.
        let mut agent = h
            .spawn(
                &cal_root,
                Rights::WRITE,
                Duration::from_secs(600),
                Quota::new(20, 0),
            )
            .unwrap();
        h.grant(&mut agent, &doc_root, Rights::READ).unwrap();

        // Within each grant it can act…
        assert!(h.execute(&mut agent, calendar, Rights::WRITE).is_ok());
        assert!(h.execute(&mut agent, document, Rights::READ).is_ok());
        // …but not outside: write on the document (read-only) is denied.
        assert!(matches!(
            h.execute(&mut agent, document, Rights::WRITE),
            Err(AgentError::NotAuthorized)
        ));
        // …and a resource it was never granted is denied entirely.
        assert!(matches!(
            h.execute(&mut agent, Resource(99), Rights::READ),
            Err(AgentError::NotAuthorized)
        ));
    }
}
