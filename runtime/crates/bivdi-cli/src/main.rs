//! Bivdi Runtime CLI — full six-primitive demo.
//!
//! Ties the object store, capability runtime, state engine, agent host, event
//! bus, and identity service together to demonstrate the decided model.

use bivdi_agent::{AgentHost, Quota};
use bivdi_cap::{CapRuntime, Lease, Resource, Right};
use bivdi_event::{Event, EventBus, Filter};
use bivdi_identity::{Identity, IdentityService, Kind};
use bivdi_object::{blake3_hash, Store};
use bivdi_runtime::Node;
use bivdi_state::{Action, DesiredState, StateEngine};
use bivdi_wasm::WasiRuntime;
use std::collections::BTreeMap;
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 && args[1] == "persist" {
        let path = args
            .get(2)
            .map(String::as_str)
            .unwrap_or("bivdi-store.json");
        demo_persistence(path);
        return;
    }
    if args.len() >= 2 && args[1] == "scenario" {
        run_agent_scenario();
        return;
    }
    if args.len() >= 2 && args[1] == "wasm" {
        demo_wasm();
        return;
    }

    println!("== Bivdi Runtime (six primitives) ==\n");

    demo_object_store();
    demo_capabilities();
    demo_state_engine();
    demo_agent_host();
    demo_event_bus();
    demo_identity();

    println!("\nBivdi — nothing has ambient authority. Everything must ask.");
}

/// Demonstrate that state survives a save/load round trip.
fn demo_persistence(path: &str) {
    println!("== Object store persistence (provisional Phase 0 format) ==\n");

    // First "run": write state.
    let store = Store::new();
    let blob = store.put_blob(b"durable greeting".to_vec());
    let cell = store.new_cell();
    let h = blake3_hash(b"version-1");
    store.cell_cas(cell, None, Some(h)).unwrap();
    let cat = store.new_catalog();
    store.catalog_put(cat, "greeting", blob).unwrap();

    let json = store.save();
    std::fs::write(path, &json).expect("write store file");
    println!("  saved {} bytes to {path}", json.len());

    // Second "run": load state from disk.
    let loaded =
        Store::load(&std::fs::read_to_string(path).expect("read store file")).expect("load store");
    println!(
        "  loaded blob   = {:?}",
        String::from_utf8_lossy(&loaded.get_blob(blob).unwrap())
    );
    println!("  loaded cell   = {:?}", loaded.cell_read(cell).unwrap());
    println!(
        "  catalog[greeting] resolves = {}",
        loaded.catalog_get(cat, "greeting").is_some()
    );
    println!();
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
    println!("  generation {g1} active");
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

    // A "calendar" resource, owned by minting a root capability.
    let calendar = Resource(42);
    let root = host.mint_root(calendar, Right::Grant);

    // Spawn an agent from the root, attenuated to read-only, 10-action quota.
    let mut agent = host
        .spawn(
            &root,
            Right::Read,
            Duration::from_secs(60),
            Quota::new(10, 1),
        )
        .unwrap();
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
    let mut small = host
        .spawn(
            &root,
            Right::Read,
            Duration::from_secs(60),
            Quota::new(1, 0),
        )
        .unwrap();
    host.execute(&mut small, calendar, Right::Read).unwrap();
    println!(
        "  quota exhausted after 1 action = {}",
        host.execute(&mut small, calendar, Right::Read).is_err()
    );

    println!("  provenance events = {}", host.provenance_len());
    println!();
}

fn demo_event_bus() {
    println!("-- Event bus: typed events + correlation identity --");
    let bus = EventBus::new();

    // A subscriber interested only in generation activations.
    let gen_sub = bus.subscribe(Filter::kind("generation_activated"));

    // Events carry a correlation id tracing one action.
    let corr = bus.new_correlation();
    bus.publish(Event::GenerationActivated {
        correlation: corr.0,
        generation: 7,
    });
    bus.publish(Event::ObjectChanged {
        correlation: corr.0,
        object: "photo-1".into(),
    });

    println!(
        "  generation subscriber received {} event(s)",
        bus.drain(gen_sub).len()
    );
    println!("  total history = {}", bus.history().len());
    println!("  correlation id = {}", corr.0);
    println!();
}

/// The RFC 0001 §3.4 scenario, run through the composed runtime node.
fn run_agent_scenario() {
    println!("== Agent scenario (RFC 0001 §3.4) ==\n");

    let mut node = Node::new(DesiredState {
        workloads: BTreeMap::new(),
    });

    // A "calendar entry" resource the operator owns.
    let calendar = Resource(42);
    let root = node.mint_root(calendar, Right::Grant);

    // Grant the agent a leased, write-scoped capability to exactly one entry.
    let agent_id = node
        .spawn_agent(
            &root,
            Right::Write,
            Duration::from_millis(150),
            Quota::new(20, 0),
        )
        .unwrap();
    println!("  agent {agent_id} spawned with a 150ms leased Write capability");

    // The agent can write (within its grant)…
    println!(
        "  allowed write = {}",
        node.agent_act(agent_id, calendar, Right::Write).is_ok()
    );

    // …but a prompt-injection "forward the mailbox" is impossible: the agent
    // holds no capability naming the mailbox, and no network flow capability.
    let mailbox = Resource(7);
    println!(
        "  mailbox read denied = {}",
        node.agent_act(agent_id, mailbox, Right::Read).is_err()
    );

    // After the lease expires, even the legitimate write is denied.
    std::thread::sleep(Duration::from_millis(200));
    println!(
        "  write after lease expiry denied = {}",
        node.agent_act(agent_id, calendar, Right::Write).is_err()
    );

    println!(
        "  event-fabric history = {} events",
        node.events.history().len()
    );
    println!();
}

/// Demonstrate the WASI runtime: compile and run a WASI module.
fn demo_wasm() {
    println!("== WASI runtime (D-004 native application format) ==\n");

    // A WASI module exporting `add(i32, i32) -> i32`.
    let add_wat = "(module (func (export \"add\") (param i32 i32) (result i32) local.get 0 local.get 1 i32.add))";
    let add_wasm = wat::parse_str(add_wat).unwrap();

    let rt = WasiRuntime::new_least(&add_wasm).unwrap();
    println!("  compiled module, exports = {:?}", rt.exports());
    println!("  add(2, 3) = {}", rt.call_i32_i32("add", 2, 3).unwrap());

    // A WASI command module (runs `_start` with least authority: stdout only).
    let cmd_wat = "(module (func (export \"_start\")))";
    let cmd_wasm = wat::parse_str(cmd_wat).unwrap();
    let cmd = WasiRuntime::new_least(&cmd_wasm).unwrap();
    cmd.run_command().unwrap();
    println!("  WASI command ran to completion (least authority)");
    println!();
}

fn demo_identity() {
    println!("-- Identity: petnames + selective disclosure --");
    let mut svc = IdentityService::new();

    let alice = Identity {
        fingerprint: [7; 32],
        claimed_name: "alice@example.com".into(),
        kind: Kind::Person,
    };
    svc.register(alice.clone());
    svc.assign_petname(&alice.fingerprint, "mom");
    svc.set_attribute(&alice.fingerprint, "birth_year", "1990");

    println!("  registered identity, claimed name hidden");
    println!("  display name = {}", svc.display_name(&alice.fingerprint));
    println!(
        "  is_adult(2026) = {} (birth year not revealed)",
        svc.is_adult(&alice.fingerprint, 2026)
    );
    println!();
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}
