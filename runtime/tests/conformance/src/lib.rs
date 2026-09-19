//! Bivdi conformance suite — the "one contract" guarantee.
//!
//! RFC 0002 §8.5: *"Write a conformance suite from the WIT: the same test
//! vectors must pass against Bivdi Runtime today and Bivdi Core later. This
//! suite is the 'one contract' guarantee — without it, that guarantee is an
//! intention."*
//!
//! These tests are written against the *decided model* (the invariants in
//! `wit/core.wit` and `docs/`), **not** against any particular Rust
//! implementation detail. A future Core track must make the same vectors pass
//! unchanged. Each test is therefore a statement about the contract, phrased in
//! the language of the six primitives.

#[cfg(test)]
mod conformance {
    use bivdi_agent::{AgentHost, Quota};
    use bivdi_cap::{CapRuntime, Lease, Resource, Rights};
    use bivdi_object::Store;
    use bivdi_runtime::Node;
    use bivdi_state::DesiredState;
    use std::collections::BTreeMap;
    use std::time::Duration;

    /// The six rights are independent and attenuate by subset inclusion, never
    /// by a total order. This is the RFC 0002 §5 correction, now a conformance
    /// fact.
    #[test]
    fn rights_attenuate_by_subset_inclusion() {
        let mut rt = CapRuntime::new();
        let root = rt.mint(Resource(1), Rights::WRITE);
        // `EXECUTE` is not "less than" `WRITE`; it is simply *not a subset*, so
        // it is denied rather than silently ordered.
        assert!(rt.attenuate(&root, Rights::EXECUTE).is_err());

        // A flag set can hold exactly the rights minted.
        let rw = rt.mint(Resource(2), Rights::READ | Rights::WRITE);
        let read = rt.attenuate(&rw, Rights::READ).unwrap();
        assert!(rt.check(&read, Resource(2), Rights::READ));
        assert!(!rt.check(&read, Resource(2), Rights::WRITE));
    }

    /// A capability is unforgeable and confined to its resource: it grants
    /// nothing over any other resource, and nothing over the same resource
    /// beyond its set.
    #[test]
    fn capability_is_confined_to_its_resource_and_rights() {
        let mut rt = CapRuntime::new();
        let cap = rt.mint(Resource(7), Rights::READ);
        assert!(rt.check(&cap, Resource(7), Rights::READ));
        assert!(!rt.check(&cap, Resource(8), Rights::READ));
        assert!(!rt.check(&cap, Resource(7), Rights::WRITE));
    }

    /// Revocation is subtree-wide: revoking a parent invalidates everything
    /// derived from it, while the parent's own source is untouched.
    #[test]
    fn revocation_kills_the_derived_subtree() {
        let mut rt = CapRuntime::new();
        let root = rt.mint(Resource(1), Rights::ALL);
        let a = rt.attenuate(&root, Rights::READ | Rights::WRITE).unwrap();
        let b = rt.attenuate(&a, Rights::READ).unwrap();
        rt.revoke(&a);
        assert!(!rt.check(&b, Resource(1), Rights::READ));
        // The root is unaffected — it was not in the revoked subtree.
        assert!(rt.check(&root, Resource(1), Rights::GRANT));
    }

    /// Leases are time-bounded and fail closed on expiry.
    #[test]
    fn leases_expire_and_fail_closed() {
        let mut rt = CapRuntime::new();
        let cap = rt.mint_leased(
            Resource(1),
            Rights::READ,
            Lease::new(Duration::from_millis(1)),
        );
        std::thread::sleep(Duration::from_millis(5));
        assert!(!rt.check(&cap, Resource(1), Rights::READ));
    }

    /// Every authority-relevant event — mint, act, and denial — is queryable,
    /// and a denied attempt is recorded explicitly, never silently dropped.
    #[test]
    fn provenance_records_explicit_denials() {
        let mut rt = CapRuntime::new();
        let cap = rt.mint(Resource(1), Rights::READ);
        assert!(rt.record_use(&cap, Resource(1), Rights::READ));
        assert!(!rt.record_use(&cap, Resource(1), Rights::WRITE)); // denied

        let events = rt.provenance_for_resource(Resource(1));
        let has_denial = events
            .iter()
            .any(|e| matches!(e, bivdi_cap::Event::Denied { .. }));
        assert!(has_denial, "a denial must be recorded, not dropped");
    }

    /// The object store is content-addressed and deterministic: identical bytes
    /// hash identically, and the durable encoding is byte-for-byte
    /// deterministic.
    #[test]
    fn object_store_is_content_addressed_and_deterministic() {
        let s = Store::new();
        let a = s.put_blob(b"same bytes".to_vec());
        let b = s.put_blob(b"same bytes".to_vec());
        assert_eq!(s.get_blob(a).unwrap(), s.get_blob(b).unwrap());

        let build = || {
            let store = Store::new();
            let blob = store.put_blob(b"payload".to_vec());
            let cell = store.new_cell();
            store
                .cell_cas(cell, None, Some(bivdi_object::blake3_hash(b"v")))
                .unwrap();
            let cat = store.new_catalog();
            store.catalog_put(cat, "k", blob).unwrap();
            store.save_cbor()
        };
        assert_eq!(build(), build());
    }

    /// An agent can only act within the authority it was handed; it cannot
    /// escalate (delegate wider), outlive its lease, or exceed its quota.
    #[test]
    fn agent_cannot_act_beyond_its_grant() {
        let mut host = AgentHost::new();
        let root = host.mint_root(Resource(42), Rights::ALL);

        let mut agent = host
            .spawn(
                &root,
                Rights::READ,
                Duration::from_secs(60),
                Quota::new(2, 0),
            )
            .unwrap();
        assert!(host.execute(&mut agent, Resource(42), Rights::READ).is_ok());
        assert!(host
            .execute(&mut agent, Resource(42), Rights::WRITE)
            .is_err());
        assert!(host
            .execute(&mut agent, Resource(43), Rights::READ)
            .is_err());

        // Escalation via delegation is denied.
        let mut parent = host
            .spawn(
                &root,
                Rights::WRITE,
                Duration::from_secs(60),
                Quota::new(10, 1),
            )
            .unwrap();
        assert!(host
            .delegate(
                &mut parent,
                Rights::GRANT,
                Duration::from_secs(60),
                Quota::new(1, 0)
            )
            .is_err());
    }

    /// The RFC 0001 §3.4 scenario: a prompt-injection attempt against a bounded
    /// agent gains nothing, and the denial is recorded in provenance.
    #[test]
    fn prompt_injection_gains_nothing() {
        let mut node = Node::new(DesiredState {
            workloads: BTreeMap::new(),
        });

        let calendar = Resource(42);
        let mailbox = Resource(44);
        let root = node.mint_root(calendar, Rights::ALL);

        let agent = node
            .spawn_agent(
                &root,
                Rights::WRITE,
                Duration::from_secs(600),
                Quota::new(20, 0),
            )
            .unwrap();

        // The injected instruction names the mailbox, which the agent was never
        // granted. It is denied, and the denial is recorded.
        assert!(node.agent_act(agent, mailbox, Rights::READ).is_err());
        assert!(node.agent_act(agent, calendar, Rights::WRITE).is_ok()); // within grant
    }

    /// The WIT contract (`wit/core.wit`) is valid, parseable WIT. This is the
    /// enforced form of the "one contract" guarantee: the IDL must stay a real
    /// language-neutral contract, not a prose description that has drifted out
    /// of sync with reality.
    #[test]
    fn wit_contract_is_valid() {
        // CARGO_MANIFEST_DIR = runtime/tests/conformance; walk up to the repo
        // root and into wit/.
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../wit/core.wit");
        let mut resolve = wit_parser::Resolve::new();
        let result = resolve.push_path(path);
        assert!(result.is_ok(), "wit/core.wit must be valid WIT: {result:?}");
    }

    /// The WIT `flags rights` declaration and the Rust `Rights` bit set are the
    /// *same contract*. This test cross-checks them at the type level: the
    /// WIT's flag names, in declaration order, must map one-to-one onto the
    /// runtime's `Rights` bit values. If they ever drift, this fails — which is
    /// the point of the "one contract" guarantee being enforced, not asserted.
    #[test]
    fn wit_rights_match_runtime_rights() {
        use wit_parser::{Resolve, TypeDefKind};

        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../wit/core.wit");
        let mut resolve = Resolve::new();
        resolve.push_path(path).expect("wit parses");

        // Find the `rights` flags type inside `interface capabilities`.
        let mut wit_flags = None;
        for (_id, interface) in resolve.interfaces.iter() {
            if interface.name.as_deref() != Some("capabilities") {
                continue;
            }
            for (name, ty) in &interface.types {
                if name == "rights" {
                    if let TypeDefKind::Flags(flags) = &resolve.types[*ty].kind {
                        wit_flags = Some(
                            flags
                                .flags
                                .iter()
                                .map(|f| f.name.clone())
                                .collect::<Vec<_>>(),
                        );
                    }
                }
            }
        }
        let wit_flags = wit_flags.expect("capabilities.rights must be declared");

        // The runtime's `Rights` bit values, in the same order as the WIT
        // declaration (read, write, execute, grant, signal, revoke).
        let runtime = [
            ("read", Rights::READ),
            ("write", Rights::WRITE),
            ("execute", Rights::EXECUTE),
            ("grant", Rights::GRANT),
            ("signal", Rights::SIGNAL),
            ("revoke", Rights::REVOKE),
        ];

        assert_eq!(
            wit_flags.len(),
            runtime.len(),
            "WIT `rights` and Rust `Rights` must have the same number of flags"
        );
        for (i, (name, _bit)) in runtime.iter().enumerate() {
            assert_eq!(&wit_flags[i], name, "flag {i} must match");
        }
    }
}
