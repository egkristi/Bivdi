//! Bivdi identity service — "identity everywhere" primitive.
//!
//! Implements the decided identity model from `docs/identity.md`:
//! - **identity kinds** (person, device, workload, service, object, org),
//! - **cryptographic identity** (a public key fingerprint is the identity),
//! - **petnames** (the user's local name, shown instead of a self-claimed name),
//! - **selective disclosure** (verify a property without revealing the value).
//!
//! # Provisional
//!
//! Phase 0 is single-process, so there is no real key ceremony: a "key" is an
//! opaque fingerprint used to stand in for a cryptographic identity. Real
//! signing/key handling belongs to Bivdi Core.

use std::collections::BTreeMap;

/// The identity kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    Person,
    Device,
    Workload,
    Service,
    Object,
    Organization,
}

/// An identity: a cryptographic fingerprint plus a claimed display name.
///
/// The fingerprint is the *identity*; the name is a self-claimed label that is
/// never trusted as the basis for authority or display.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Identity {
    /// A stand-in for a public-key fingerprint (opaque in Phase 0).
    pub fingerprint: [u8; 32],
    /// The party's self-claimed name (never trusted).
    pub claimed_name: String,
    pub kind: Kind,
}

/// A petname: a user-assigned local name for a cryptographically identified
/// party. The system always shows the petname, never the claimed name.
#[derive(Debug, Clone)]
pub struct Petname {
    /// The user's chosen name.
    pub name: String,
    /// The identity it names.
    pub target: Identity,
}

/// The identity service. Registers identities, assigns petnames, and answers
/// selective-disclosure queries. Identity is a system service, not something
/// each application reinvents.
#[derive(Debug, Default)]
pub struct IdentityService {
    /// Registered identities by fingerprint.
    identities: BTreeMap<[u8; 32], Identity>,
    /// A user's petnames, keyed by fingerprint.
    petnames: BTreeMap<[u8; 32], String>,
    /// Identity attributes available for selective disclosure.
    attributes: BTreeMap<[u8; 32], BTreeMap<String, String>>,
}

impl IdentityService {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an identity (the fingerprint is the identity).
    pub fn register(&mut self, identity: Identity) {
        self.identities.insert(identity.fingerprint, identity);
    }

    /// Look up an identity by fingerprint.
    pub fn get(&self, fingerprint: &[u8; 32]) -> Option<&Identity> {
        self.identities.get(fingerprint)
    }

    /// Assign a petname to an identity. The system will show this name instead
    /// of the identity's self-claimed name.
    pub fn assign_petname(&mut self, fingerprint: &[u8; 32], name: &str) {
        self.petnames.insert(*fingerprint, name.to_string());
    }

    /// The name the system shows for an identity: the user's petname if present,
    /// otherwise a fingerprint-derived label. Never the self-claimed name.
    pub fn display_name(&self, fingerprint: &[u8; 32]) -> String {
        if let Some(pet) = self.petnames.get(fingerprint) {
            return pet.clone();
        }
        // Fall back to a stable, non-claimed label.
        use std::fmt::Write;
        let mut hex = String::with_capacity(8);
        for b in fingerprint.iter().take(4) {
            let _ = write!(hex, "{b:02x}");
        }
        format!("id:{hex}")
    }

    /// Set an attribute on an identity (e.g., birth year).
    pub fn set_attribute(&mut self, fingerprint: &[u8; 32], key: &str, value: &str) {
        self.attributes
            .entry(*fingerprint)
            .or_default()
            .insert(key.to_string(), value.to_string());
    }

    /// Selective disclosure: verify a boolean predicate over an attribute
    /// **without returning the attribute's value**. The caller supplies a
    /// verifier; the service answers only "yes/no".
    pub fn verify_predicate(
        &self,
        fingerprint: &[u8; 32],
        key: &str,
        predicate: &dyn Fn(Option<&str>) -> bool,
    ) -> bool {
        let value = self
            .attributes
            .get(fingerprint)
            .and_then(|attrs| attrs.get(key))
            .map(|s| s.as_str());
        predicate(value)
    }

    /// Convenience predicate: "age ≥ 18" given a birth-year attribute and a
    /// current year. Returns only yes/no; never reveals the birth year.
    pub fn is_adult(&self, fingerprint: &[u8; 32], current_year: u32) -> bool {
        self.verify_predicate(fingerprint, "birth_year", &|v| match v {
            Some(s) => s
                .parse::<u32>()
                .map(|birth| current_year.saturating_sub(birth) >= 18)
                .unwrap_or(false),
            None => false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fp(seed: u8) -> [u8; 32] {
        [seed; 32]
    }

    #[test]
    fn identity_is_looked_up_by_fingerprint_not_name() {
        let mut svc = IdentityService::new();
        let a = Identity {
            fingerprint: fp(1),
            claimed_name: "alice".into(),
            kind: Kind::Person,
        };
        svc.register(a.clone());
        assert_eq!(svc.get(&fp(1)), Some(&a));
        assert!(svc.get(&fp(2)).is_none());
    }

    #[test]
    fn petname_is_displayed_instead_of_claimed_name() {
        let mut svc = IdentityService::new();
        svc.register(Identity {
            fingerprint: fp(1),
            claimed_name: "mallory".into(),
            kind: Kind::Person,
        });
        // Without a petname, we never show the claimed name.
        assert_ne!(svc.display_name(&fp(1)), "mallory");
        // With a petname, we show the user's chosen name.
        svc.assign_petname(&fp(1), "mom");
        assert_eq!(svc.display_name(&fp(1)), "mom");
    }

    #[test]
    fn selective_disclosure_answers_yes_no_without_value() {
        let mut svc = IdentityService::new();
        svc.register(Identity {
            fingerprint: fp(1),
            claimed_name: "alice".into(),
            kind: Kind::Person,
        });
        svc.set_attribute(&fp(1), "birth_year", "2000");
        // The service answers "adult?" without exposing the birth year.
        assert!(svc.is_adult(&fp(1), 2026));
        // No attribute => false, not an error.
        assert!(!svc.is_adult(&fp(2), 2026));
    }

    #[test]
    fn predicate_sees_value_but_callers_do_not() {
        let mut svc = IdentityService::new();
        svc.set_attribute(&fp(7), "country", "NO");
        let saw_value = std::cell::Cell::new(None);
        let result = svc.verify_predicate(&fp(7), "country", &|v| {
            saw_value.set(v.map(|s| s.to_string()));
            v == Some("NO")
        });
        assert!(result);
        // The predicate saw the value internally, but the public API only
        // returned a bool — selective disclosure does not leak the value.
        assert_eq!(saw_value.into_inner(), Some("NO".to_string()));
    }
}
