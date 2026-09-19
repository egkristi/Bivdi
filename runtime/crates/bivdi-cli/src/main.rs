//! Bivdi Runtime CLI — Phase 0 + Phase 1 (agent host) demo.
//!
//! Ties the object store, capability runtime, state engine, and agent host
//! together to demonstrate the decided model.

use bivdi_agent::{AgentHost, Quota};
use bivdi_cap::{CapRuntime, Lease, Resource, Right};
use bivdi_object::{blake3_hash, Store};
use bivdi_state::{Action, DesiredState, StateEngine};
use std::collections::BTreeMap;
use std::time::Duration;

fn main() {
    println!("== Bivdi Runtime (Phase 0 + Phase 1 agent host) ==\n");

    demo_object_store();
    demo_capabilities();
    demo_state_engine();
    demo_agent_host();

    println!("\nBivdi — nothing has ambient authority. Everything must ask.");
}

fn demo_object_store() {
    println!("-- Object store: content addressing + CAS --");
    let store = Store::new();
    let blob = store.put_blob(b"hello, bivdi".to_vec());
    let hash = blake3_hash(b"hello, bivdi");
    println!("  stored blob, content hash = {}", hex(&hash));

    let cell = store.new_cell();
    println!(
        "  cell initial value = {:?}",
        store.cell_read(cell).unwrap()
    );

    // CAS loop: the only mutation primitive.
    let v1 = blake3_hash(b"version-1");
    store.cell_cas(cell, None, Some(v1)).unwrap();
    match store.cell_cas(cell, None, Some(v1)) {
        Ok(()) => unreachable!(),
        Err(e) => println!("  stale CAS detected, actual = {:?}", e.actual),
    }
    let v2 = blake3_hash(b"version-2");
    store.cell_cas(cell, Some(v1), Some(v2)).unwrap();
    println!("  cell after retry = {:?}", store.cell_read(cell).unwrap());

    let catalog = store.new_catalog();
    store.catalog_put(catalog, "greeting", blob).unwrap();
    println!(
        "  catalog[\"greeting\"] resolves = {}",
        store.catalog_get(catalog, "greeting").is_some()
    );
    println!();
}

fn demo_capabilities() {
    println!("-- Capability runtime: attenuate + revoke + lease --");
    let mut rt = CapRuntime::new();

    // A resource standing in for a calendar object.
    let calendar = Resource(42);
    let grant = rt.mint(calendar, Right::Grant);

    // Attenuate to read-only. Widening is denied by construction.
    let read_only = rt.attenuate(&grant, Right::Read).unwrap();
    println!("  attenuated to Read: ok");
    let widening = rt.attenuate(&read_only, Right::Write);
    println!("  widening Read -> Write denied = {}", widening.is_err());

    println!(
        "  read_only can read = {}",
        rt.check(&read_only, calendar, Right::Read)
    );
    println!(
        "  read_only cannot write = {}",
        !rt.check(&read_only, calendar, Right::Write)
    );

    // Revoke the read-only subtree.
    rt.revoke(&read_only);
    println!(
        "  after revoke, read_only still valid = {}",
        rt.check(&read_only, calendar, Right::Read)
    );

    // A lease that expires.
    let leased = rt.mint_leased(
        Resource(7),
        Right::Read,
        Lease::new(Duration::from_millis(1)),
    );
    std::thread::sleep(Duration::from_millis(5));
    println!(
        "  expired lease still valid = {}",
        rt.check(&leased, Resource(7), Right::Read)
    );

    println!("  provenance events = {}", rt.provenance().len());
    println!();
}

fn demo_state_engine() {
    println!("-- State engine: desired state + reconciliation + generations --");
    let initial = DesiredState {
        workloads: BTreeMap::from([("web".to_string(), 1)]),
    };
    let engine = StateEngine::new(initial);

    // Apply a new generation: scale web to 3.
    let g1 = engine.apply(DesiredState {
        workloads: BTreeMap::from([("web".to_string(), 3)]),
    });
    engine.observe(BTreeMap::from([("web".to_string(), 1)]));
    println!("  generation {} active", g1);
    println!("  reconcile actions = {:?}", engine.reconcile());

    // Rollback is a pointer change.
    let prev = engine.rollback().unwrap();
    println!(
        "  rolled back to generation {} (desired web = {:?})",
        prev,
        engine.active_desired().workloads.get("web")
    );

    // Show the actions vector type is used.
    let _unused: Vec<Action> = engine.reconcile();
    println!();
}

fn demo_agent_host() {
    println!("-- Agent host: constrained, quota-bound, non-escalating --");
    let mut host = AgentHost::new();

    // A "calendar" resource.
    let calendar = Resource(42);

    // Spawn an agent with read-only access, a 10-action quota, 1 delegation.
    let mut agent = host.spawn(
        calendar,
        Right::Read,
        Duration::from_secs(60),
        Quota::new(10, 1),
    );
    println!("  spawned agent {} with Read over resource 42", agent.id);

    // It can read…
    println!(
        "  agent can read = {}",
        host.execute(&mut agent, calendar, Right::Read).is_ok()
    );
    // …but not write (prompt-injection cannot grant write).
    println!(
        "  agent cannot write = {}",
        host.execute(&mut agent, calendar, Right::Write).is_err()
    );

    // It cannot delegate a wider right than it holds.
    let escalation = host.delegate(
        &mut agent,
        Right::Write,
        Duration::from_secs(60),
        Quota::new(1, 0),
    );
    println!("  delegation cannot escalate = {}", escalation.is_err());

    // A read-only agent with a tiny quota: exhaustion is predictable.
    let mut small = host.spawn(
        calendar,
        Right::Read,
        Duration::from_secs(60),
        Quota::new(1, 0),
    );
    host.execute(&mut small, calendar, Right::Read).unwrap();
    println!(
        "  quota exhausted after 1 action = {}",
        host.execute(&mut small, calendar, Right::Read).is_err()
    );

    println!("  provenance events = {}", host.provenance_len());
    println!();
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
