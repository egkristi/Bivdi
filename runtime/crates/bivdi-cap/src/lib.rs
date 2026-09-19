//! Bivdi capability runtime — Phase 0, in-process.
//!
//! Implements the decided capability properties from `docs/capabilities.md`:
//! - **unforgeable** (opaque ids in-process; a stand-in for kernel enforcement),
//! - **transferable**, **attenuable** (subset-inclusion attenuation only),
//! - **revocable** (a revoke invalidates the whole derived subtree),
//! - **leases** (time-bounded grants),
//! - **provenance** (an append-only record of authority-relevant events).
//!
//! # Provisional
//!
//! Phase 0 is single-process on Linux, so unforgeability is simulated with
//! opaque ids rather than enforced by a kernel. The durable-token tier is out
//! of scope here (see `docs/token-format.md`).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

/// Rights over a resource, as a flag set (RFC 0002 §5). The six rights named in
/// `ARCHITECTURE.md` §4.2 — read, write, execute, grant, signal, revoke — are
/// *independent*; `EXECUTE` is not "less than" `WRITE`. Attenuation is
/// **subset inclusion**: a derived capability holds a subset of its source's
/// rights, never a right the source did not hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Rights(u8);

impl Rights {
    pub const READ: Rights = Rights(0b00_0001);
    pub const WRITE: Rights = Rights(0b00_0010);
    pub const EXECUTE: Rights = Rights(0b00_0100);
    pub const GRANT: Rights = Rights(0b00_1000);
    pub const SIGNAL: Rights = Rights(0b01_0000);
    pub const REVOKE: Rights = Rights(0b10_0000);

    /// No rights.
    pub const NONE: Rights = Rights(0);
    /// Every right.
    pub const ALL: Rights = Rights(0b11_1111);

    /// `true` if `self` is a superset of `other` (holds every right in `other`).
    pub fn contains(self, other: Rights) -> bool {
        self.0 & other.0 == other.0
    }

    /// The rights common to both sets.
    pub fn intersection(self, other: Rights) -> Rights {
        Rights(self.0 & other.0)
    }

    /// `true` if no right is held.
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl std::ops::BitOr for Rights {
    type Output = Rights;
    fn bitor(self, rhs: Rights) -> Rights {
        Rights(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for Rights {
    type Output = Rights;
    fn bitand(self, rhs: Rights) -> Rights {
        Rights(self.0 & rhs.0)
    }
}

/// A resource a capability points at. In Phase 0 a resource is a name-scoped
/// opaque id; in Bivdi Core it is a kernel object reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Resource(pub u64);

/// A capability: an unforgeable handle granting `right` over `resource`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Capability {
    id: u64,
    resource: Resource,
    right: Rights,
}

impl Capability {
    pub fn resource(&self) -> Resource {
        self.resource
    }

    pub fn right(&self) -> Rights {
        self.right
    }
}

/// A lease: an optional time bound on a capability.
#[derive(Debug, Clone, Copy)]
pub struct Lease {
    expires_at: Instant,
}

impl Lease {
    pub fn new(duration: Duration) -> Self {
        Self {
            expires_at: Instant::now() + duration,
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }

    /// The remaining duration before expiry (zero if expired). Used to clamp a
    /// derived lease so it cannot outlive the parent it was delegated from.
    pub fn remaining(&self) -> Duration {
        self.expires_at.saturating_duration_since(Instant::now())
    }
}

/// A provenance event (authority-relevant only; never content).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    Minted {
        cap: u64,
        resource: Resource,
        right: Rights,
    },
    Attenuated {
        from: u64,
        to: u64,
        right: Rights,
    },
    Revoked {
        cap: u64,
    },
    /// An authority-relevant action was performed using a capability.
    Acted {
        cap: u64,
        resource: Resource,
        right: Rights,
    },
    /// An action was attempted but the capability did not grant the requested
    /// right (or the lease had expired). Recording denials explicitly is what
    /// makes RFC 0001 §3.4 clause 3 testable: every attempt outside the grant
    /// leaves a denial in the log, rather than vanishing silently.
    Denied {
        cap: u64,
        resource: Resource,
        right: Rights,
    },
}

impl Event {
    /// The resource this event concerns, if any. Mint, Act, and Denied carry
    /// the resource; Attenuated and Revoked carry only capability ids.
    pub fn resource(&self) -> Option<Resource> {
        match self {
            Event::Minted { resource, .. }
            | Event::Acted { resource, .. }
            | Event::Denied { resource, .. } => Some(*resource),
            Event::Attenuated { .. } | Event::Revoked { .. } => None,
        }
    }
}

/// Hash one log entry: `BLAKE3(prev_hash || serialized_event)`. The event is
/// serialized with a deterministic, self-describing format (serde_json with
/// sorted keys would be ideal, but the `Event` struct's field order is fixed
/// and stable, so `serde_json::to_vec` is deterministic here).
fn hash_entry(prev: Hash, event: &Event) -> Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&prev);
    hasher.update(&serde_json::to_vec(event).expect("provenance event serializes"));
    *hasher.finalize().as_bytes()
}

/// The capability runtime. Mints, attenuates, and revokes capabilities while
/// recording an append-only, hash-chained provenance log.
///
/// Each entry's hash commits to the previous entry's hash and to the entry's
/// own content, so a tampered or reordered log breaks the chain (H4). The
/// genesis hash is a fixed constant, not derived from a mutable state.
#[derive(Debug, Default)]
pub struct CapRuntime {
    next_id: u64,
    caps: BTreeMap<u64, (Capability, Option<Lease>)>,
    /// Parent link for subtree revocation.
    parent: BTreeMap<u64, u64>,
    /// The append-only provenance log and its hash chain. `chain[i]` commits
    /// to `chain[i-1]` and to `provenance[i]`.
    provenance: Vec<Event>,
    chain: Vec<Hash>,
}

/// The chain hash type (BLAKE3-256, matching the object store's provisional
/// content hash).
pub type Hash = [u8; 32];

/// The fixed genesis hash for an empty log. A constant, so an empty log has a
/// well-defined, non-forgeable root.
const GENESIS: Hash = [0x42; 32];

impl CapRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an event and advance the hash chain. Returns the new head hash.
    fn record(&mut self, event: Event) -> Hash {
        let prev = self.chain.last().copied().unwrap_or(GENESIS);
        let head = hash_entry(prev, &event);
        self.provenance.push(event);
        self.chain.push(head);
        head
    }

    /// The head hash of the chain (the tamper-evident root of the log).
    pub fn chain_head(&self) -> Hash {
        self.chain.last().copied().unwrap_or(GENESIS)
    }

    /// Verify the chain is intact: recompute every hash from the genesis and
    /// assert it matches the recorded chain. `true` iff no entry has been
    /// altered, removed, or reordered.
    pub fn verify_chain(&self) -> bool {
        let mut prev = GENESIS;
        for (event, recorded) in self.provenance.iter().zip(&self.chain) {
            if hash_entry(prev, event) != *recorded {
                return false;
            }
            prev = *recorded;
        }
        true
    }

    /// Mint a new root capability with the given rights over a resource.
    pub fn mint(&mut self, resource: Resource, right: Rights) -> Capability {
        let id = self.alloc();
        self.caps.insert(
            id,
            (
                Capability {
                    id,
                    resource,
                    right,
                },
                None,
            ),
        );
        self.record(Event::Minted {
            cap: id,
            resource,
            right,
        });
        Capability {
            id,
            resource,
            right,
        }
    }

    /// Mint a new root capability with a lease.
    pub fn mint_leased(&mut self, resource: Resource, right: Rights, lease: Lease) -> Capability {
        let cap = self.mint(resource, right);
        self.caps.get_mut(&cap.id).unwrap().1 = Some(lease);
        cap
    }

    /// Attenuate `source` to a subset of its rights over the same resource.
    /// Attenuation never requires authority.
    pub fn attenuate(
        &mut self,
        source: &Capability,
        right: Rights,
    ) -> Result<Capability, CapError> {
        let (resource, held_right, lease) = {
            let (src, lease) = self.caps.get(&source.id).ok_or(CapError::NotHeld)?;
            (src.resource, src.right, *lease)
        };
        // Attenuation is subset inclusion: the derived rights must be a subset
        // of the held rights. Any right the source did not hold is a widening.
        if !held_right.contains(right) {
            return Err(CapError::WideningDenied);
        }
        let id = self.alloc();
        let derived = Capability {
            id,
            resource,
            right,
        };
        self.caps.insert(id, (derived, lease));
        self.parent.insert(id, source.id);
        self.record(Event::Attenuated {
            from: source.id,
            to: id,
            right,
        });
        Ok(derived)
    }

    /// Check that a capability is currently held, unexpired, and grants at
    /// least `required` right over `resource`.
    pub fn check(&self, cap: &Capability, resource: Resource, required: Rights) -> bool {
        match self.caps.get(&cap.id) {
            Some((held, lease)) => {
                if let Some(l) = lease {
                    if l.is_expired() {
                        return false;
                    }
                }
                held.resource == resource && held.right.contains(required)
            }
            None => false,
        }
    }

    /// The lease currently attached to a capability, if any.
    pub fn lease_of(&self, cap: &Capability) -> Option<Lease> {
        self.caps.get(&cap.id).and_then(|(_, lease)| *lease)
    }

    /// Revoke a capability and its entire derived subtree (transitively).
    pub fn revoke(&mut self, cap: &Capability) {
        let mut to_remove = BTreeSet::new();
        self.collect_subtree(cap.id, &mut to_remove);
        for id in &to_remove {
            self.caps.remove(id);
            self.parent.remove(id);
            self.record(Event::Revoked { cap: *id });
        }
    }

    /// Record that `cap` was used to exercise `right` over `resource`.
    /// On success, records an `Acted` event and returns `true`. On failure
    /// (not held, expired, or right not granted), records an explicit `Denied`
    /// event and returns `false` — a denial is itself authority-relevant and
    /// must never vanish from the log (RFC 0001 §3.4 clause 3).
    pub fn record_use(&mut self, cap: &Capability, resource: Resource, right: Rights) -> bool {
        if self.check(cap, resource, right) {
            self.record(Event::Acted {
                cap: cap.id,
                resource,
                right,
            });
            true
        } else {
            self.record(Event::Denied {
                cap: cap.id,
                resource,
                right,
            });
            false
        }
    }

    fn collect_subtree(&self, root: u64, out: &mut BTreeSet<u64>) {
        if out.insert(root) {
            for (child, parent) in &self.parent {
                if *parent == root {
                    self.collect_subtree(*child, out);
                }
            }
        }
    }

    /// The append-only provenance log.
    pub fn provenance(&self) -> &[Event] {
        &self.provenance
    }

    /// Query the provenance log for every authority event naming `resource`, in
    /// append order. This is the operator-facing query — "what touched this
    /// object, and what authorised each touch?" — a structured answer, not a
    /// raw log dive.
    pub fn provenance_for_resource(&self, resource: Resource) -> Vec<Event> {
        self.provenance
            .iter()
            .filter(|e| e.resource() == Some(resource))
            .cloned()
            .collect()
    }

    /// Query the provenance log for every event in one capability's authority
    /// chain — its mint, the attenuations into and out of it, its uses, its
    /// denials, and its revocation — in append order.
    pub fn provenance_for_capability(&self, cap: u64) -> Vec<Event> {
        self.provenance
            .iter()
            .filter(|e| match e {
                Event::Minted { cap: c, .. }
                | Event::Revoked { cap: c }
                | Event::Acted { cap: c, .. }
                | Event::Denied { cap: c, .. } => *c == cap,
                Event::Attenuated { from, to, .. } => *from == cap || *to == cap,
            })
            .cloned()
            .collect()
    }

    fn alloc(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

/// Capability runtime errors.
#[derive(Debug)]
pub enum CapError {
    NotHeld,
    WideningDenied,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attenuation_never_widens() {
        let mut rt = CapRuntime::new();
        let root = rt.mint(Resource(1), Rights::READ);
        assert!(rt.attenuate(&root, Rights::READ).is_ok());
        assert!(matches!(
            rt.attenuate(&root, Rights::WRITE),
            Err(CapError::WideningDenied)
        ));
    }

    #[test]
    fn attenuation_is_subset_inclusion_not_ordering() {
        // EXECUTE is not "less than" WRITE: a Write holder cannot attenuate to
        // Execute because Execute is not a subset of Write (RFC 0002 §5).
        let mut rt = CapRuntime::new();
        let root = rt.mint(Resource(1), Rights::WRITE);
        assert!(matches!(
            rt.attenuate(&root, Rights::EXECUTE),
            Err(CapError::WideningDenied)
        ));
        // But WRITE|READ can attenuate to READ (a subset).
        let rw = rt.mint(Resource(2), Rights::WRITE | Rights::READ);
        assert!(rt.attenuate(&rw, Rights::READ).is_ok());
    }

    #[test]
    fn check_requires_right_and_resource() {
        let mut rt = CapRuntime::new();
        let root = rt.mint(Resource(7), Rights::READ);
        let derived = rt.attenuate(&root, Rights::READ).unwrap();
        assert!(rt.check(&derived, Resource(7), Rights::READ));
        assert!(!rt.check(&derived, Resource(7), Rights::WRITE));
        assert!(!rt.check(&derived, Resource(8), Rights::READ));
    }

    #[test]
    fn revoke_kills_subtree() {
        let mut rt = CapRuntime::new();
        let root = rt.mint(Resource(1), Rights::ALL);
        // A valid subset chain: ALL -> READ|WRITE -> READ.
        let a = rt.attenuate(&root, Rights::READ | Rights::WRITE).unwrap();
        let b = rt.attenuate(&a, Rights::READ).unwrap();
        assert!(rt.check(&b, Resource(1), Rights::READ));
        rt.revoke(&a);
        assert!(!rt.check(&b, Resource(1), Rights::READ));
        // Root (parent) is unaffected.
        assert!(rt.check(&root, Resource(1), Rights::GRANT));
    }

    #[test]
    fn leases_expire() {
        let mut rt = CapRuntime::new();
        let short = rt.mint_leased(
            Resource(1),
            Rights::READ,
            Lease::new(Duration::from_millis(1)),
        );
        std::thread::sleep(Duration::from_millis(5));
        assert!(!rt.check(&short, Resource(1), Rights::READ));
    }

    #[test]
    fn provenance_is_append_only() {
        let mut rt = CapRuntime::new();
        let root = rt.mint(Resource(1), Rights::READ);
        let n0 = rt.provenance().len();
        let _ = rt.attenuate(&root, Rights::READ).unwrap();
        assert_eq!(rt.provenance().len(), n0 + 1);
    }

    #[test]
    fn provenance_is_queryable_by_resource() {
        let mut rt = CapRuntime::new();
        let a = rt.mint(Resource(1), Rights::WRITE);
        let b = rt.mint(Resource(2), Rights::READ);
        rt.record_use(&a, Resource(1), Rights::WRITE);
        rt.record_use(&b, Resource(2), Rights::READ);

        let for_one = rt.provenance_for_resource(Resource(1));
        assert!(!for_one.is_empty());
        assert!(for_one.iter().all(|e| e.resource() == Some(Resource(1))));
        assert_eq!(rt.provenance_for_resource(Resource(2)).len(), 2); // mint + act
        assert!(rt.provenance_for_resource(Resource(99)).is_empty());
    }

    #[test]
    fn provenance_chain_verifies_and_detects_tampering() {
        let mut rt = CapRuntime::new();
        let root = rt.mint(Resource(1), Rights::READ);
        rt.record_use(&root, Resource(1), Rights::READ);
        assert!(rt.verify_chain(), "an untampered chain must verify");

        // Tamper with a recorded entry: the chain must fail verification.
        rt.chain[0] = [0xAA; 32];
        assert!(!rt.verify_chain(), "a tampered chain must not verify");
    }

    #[test]
    fn empty_log_verifies_against_genesis() {
        let rt = CapRuntime::new();
        assert_eq!(rt.chain_head(), GENESIS);
        assert!(rt.verify_chain());
    }
}
