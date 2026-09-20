# Bivdi — Getting Started

**Status:** Current as of 2026-09-20. The Bivdi Runtime is the product (`D-003`); this guide takes you from a fresh checkout to running the agent-isolation scenario and querying provenance.

## 1. Prerequisites

- **Rust** 1.88 or later (MSRV). The workspace pins the toolchain in `runtime/rust-toolchain.toml`; `rustup` will pick it up automatically.
- A Linux host. The sandbox (`bivdi-sandbox`) uses seccomp and Landlock; both degrade gracefully (best-effort) on hosts that forbid them, and the CLI reports the state honestly rather than overclaiming.

## 2. Build and test

```sh
cd runtime
cargo build --workspace
cargo test --workspace        # ~70 tests, including the conformance suite
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo audit                   # security advisories (0 vulnerabilities expected)
```

## 3. Run the demo

```sh
cargo run -p bivdi-cli                    # full six-primitive demo
cargo run -p bivdi-cli -- scenario        # RFC 0001 §3.4 agent exit-gate scenario
cargo run -p bivdi-cli -- persist <path>  # object-store save/load round trip to disk
cargo run -p bivdi-cli -- wasm            # compile + run a WASI module
cargo run -p bivdi-cli -- sandbox         # report seccomp/Landlock state
```

### What `scenario` shows

An agent is granted a leased, write-scoped capability to exactly one calendar
entry (and, in the full test, read access to exactly one document). The document
contains an instruction directing the agent to forward a mailbox. The output
shows:

- the legitimate write succeeds;
- the prompt-injection "read the mailbox" is **denied**;
- after the lease expires, even the legitimate write is denied;
- the **detailed authority provenance** — `Minted` / `Attenuated` / `Acted` /
  `Denied` events, each with its resource and rights — so you can answer "what
  did this agent touch, and what authorised each touch?" from the log alone.

## 4. The model in one screen

```
IDENTITY + CAPABILITY + OBJECT + EVENT + DESIRED STATE + RESOURCE
```

- **No ambient authority** — a component's authority is exactly the capabilities it was handed.
- **Objects over paths** — a typed, content-addressed object store; the filesystem is a compatibility view.
- **Declarative state** — desired state, reconciled; immutable generations; rollback is a pointer change.
- **Events over polling** — structured, first-class events.
- **AI without unrestricted authority** — agents are capability-constrained, time-limited, quota-bound, and fully provenance-logged.

See [`README.md`](../README.md), [`SPEC.md`](../SPEC.md), and the [`docs/`](.) for the full model and the decided/proposed boundary.

## 5. Where the implementation stands

The Runtime implements the six primitives (object, capability, state, event,
identity, agent) plus the WASI host, networking model, deterministic CBOR
persistence (atomic + fsync), hash-chained provenance, and the seccomp/Landlock
sandbox. The remaining Milestone A/C work — WIT bindings wiring (the SDK) and
per-endpoint WASI egress mediation — is tracked in [`ROADMAP.md`](../ROADMAP.md).

## 6. Contributing

Everything is issue-driven and CI-gated: create the issue, branch with its ID,
commit with `git commit -s` (DCO), open a linked MR, and let the five required
CI checks gate the merge. See [`CONTRIBUTING.md`](../CONTRIBUTING.md).

---

*Bivdi — nothing has ambient authority. Everything must ask.*
