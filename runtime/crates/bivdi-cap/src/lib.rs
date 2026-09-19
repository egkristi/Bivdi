//! Bivdi capability runtime — Phase 0, in-process.
//!
//! Implements the decided capability properties from `docs/capabilities.md`:
//! - **unforgeable** (opaque ids in-process; a stand-in for kernel enforcement),
//! - **transferable**, **attenuable** (monotonically narrowing rights),
//! - **revocable** (a revoke invalidates the whole derived subtree),
//! - **leases** (time-bounded grants),
//! - **provenance** (an append-only record of authority-relevant events).
//!
//! # Provisional
//!
//! Phase 0 is single-process on Linux, so unforgeability is simulated with
//! opaque ids rather than enforced by a kernel. The durable-token tier is out
//! of scope here (see `docs/token-format.md`).

use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

/// Rights over a resource. Ordering is by inclusion: `READ < WRITE < GRANT`.
/// Rights can only ever be narrowed (attenuated), never widened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Right {
    Read,
    Write,
    Grant,
}

/// A resource a capability points at. In Phase 0 a resource is a name-scoped
/// opaque id; in Bivdi Core it is a kernel object reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Resource(pub u64);

/// A capability: an unforgeable handle granting `right` over `resource`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Capability {
    id: u64,
    resource: Resource,
    right: Right,
}

impl Capability {
    pub fn resource(&self) -> Resource {
        self.resource
    }

    pub fn right(&self) -> Right {
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
}

/// A provenance event (authority-relevant only; never content).
#[derive(Debug, Clone)]
pub enum Event {
    Minted {
        cap: u64,
        resource: Resource,
        right: Right,
    },
    Attenuated {
        from: u64,
        to: u64,
        right: Right,
    },
    Revoked {
        cap: u64,
    },
}

/// The capability runtime. Mints, attenuates, and revokes capabilities while
/// recording an append-only provenance log.
#[derive(Debug, Default)]
pub struct CapRuntime {
    next_id: u64,
    caps: BTreeMap<u64, (Capability, Option<Lease>)>,
    /// Parent link for subtree revocation.
    parent: BTreeMap<u64, u64>,
    provenance: Vec<Event>,
}

impl CapRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mint a new root capability with the given right over a resource.
    pub fn mint(&mut self, resource: Resource, right: Right) -> Capability {
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
        self.provenance.push(Event::Minted {
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
    pub fn mint_leased(&mut self, resource: Resource, right: Right, lease: Lease) -> Capability {
        let cap = self.mint(resource, right);
        self.caps.get_mut(&cap.id).unwrap().1 = Some(lease);
        cap
    }

    /// Attenuate `source` to a strictly weaker (or equal) right over the same
    /// resource. Attenuation never requires authority.
    pub fn attenuate(&mut self, source: &Capability, right: Right) -> Result<Capability, CapError> {
        let (resource, held_right, lease) = {
            let (src, lease) = self.caps.get(&source.id).ok_or(CapError::NotHeld)?;
            (src.resource, src.right, *lease)
        };
        if right > held_right {
            // Rights are ordered READ < WRITE < GRANT; a larger right is a
            // widening, which is disallowed.
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
        self.provenance.push(Event::Attenuated {
            from: source.id,
            to: id,
            right,
        });
        Ok(derived)
    }

    /// Check that a capability is currently held, unexpired, and grants at
    /// least `required` right over `resource`.
    pub fn check(&self, cap: &Capability, resource: Resource, required: Right) -> bool {
        match self.caps.get(&cap.id) {
            Some((held, lease)) => {
                if let Some(l) = lease {
                    if l.is_expired() {
                        return false;
                    }
                }
                held.resource == resource && held.right >= required
            }
            None => false,
        }
    }

    /// Revoke a capability and its entire derived subtree (transitively).
    pub fn revoke(&mut self, cap: &Capability) {
        let mut to_remove = BTreeSet::new();
        self.collect_subtree(cap.id, &mut to_remove);
        for id in &to_remove {
            self.caps.remove(id);
            self.parent.remove(id);
            self.provenance.push(Event::Revoked { cap: *id });
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
        let root = rt.mint(Resource(1), Right::Read);
        assert!(rt.attenuate(&root, Right::Read).is_ok());
        assert!(matches!(
            rt.attenuate(&root, Right::Write),
            Err(CapError::WideningDenied)
        ));
    }

    #[test]
    fn check_requires_right_and_resource() {
        let mut rt = CapRuntime::new();
        let root = rt.mint(Resource(7), Right::Read);
        let derived = rt.attenuate(&root, Right::Read).unwrap();
        assert!(rt.check(&derived, Resource(7), Right::Read));
        assert!(!rt.check(&derived, Resource(7), Right::Write));
        assert!(!rt.check(&derived, Resource(8), Right::Read));
    }

    #[test]
    fn revoke_kills_subtree() {
        let mut rt = CapRuntime::new();
        let root = rt.mint(Resource(1), Right::Grant);
        let a = rt.attenuate(&root, Right::Write).unwrap();
        let b = rt.attenuate(&a, Right::Read).unwrap();
        assert!(rt.check(&b, Resource(1), Right::Read));
        rt.revoke(&a);
        assert!(!rt.check(&b, Resource(1), Right::Read));
        // Root (parent) is unaffected.
        assert!(rt.check(&root, Resource(1), Right::Grant));
    }

    #[test]
    fn leases_expire() {
        let mut rt = CapRuntime::new();
        let short = rt.mint_leased(
            Resource(1),
            Right::Read,
            Lease::new(Duration::from_millis(1)),
        );
        std::thread::sleep(Duration::from_millis(5));
        assert!(!rt.check(&short, Resource(1), Right::Read));
    }

    #[test]
    fn provenance_is_append_only() {
        let mut rt = CapRuntime::new();
        let root = rt.mint(Resource(1), Right::Read);
        let n0 = rt.provenance().len();
        let _ = rt.attenuate(&root, Right::Read).unwrap();
        assert_eq!(rt.provenance().len(), n0 + 1);
    }
}
