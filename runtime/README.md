# Bivdi Runtime (Phase 0)

The **Bivdi Runtime** is the Linux-hosted implementation of the Bivdi model — the first of the two parallel tracks (`D-003`). It proves the object, capability, and state model on Linux before the model is ported to a microkernel.

> **The kernel is not the product.** The product is the object, capability, and state model. This runtime is the model in executable form.

## Status

Phase 0 **complete**, and the Runtime half of Phase 1 **complete**. All six
primitives (object, capability, state, event, identity, agent) are implemented
with persistence, composed into an integrated node, and exercised by a WASI
runtime and CLI demo. **This is a userspace prototype on Linux, not an operating
system** — there is no kernel, no boot, and no VM guest. The microkernel half of
Phase 1 is blocked on `P-001`.

## Crates

| Crate | What it implements |
|---|---|
| `bivdi-object` | The **object store**: content-addressed blobs, CAS cells, catalogs, save/load. |
| `bivdi-cap` | The **capability runtime**: mint, attenuate, revoke (subtree), leases, provenance. |
| `bivdi-state` | The **state engine**: desired state, reconciliation, immutable generations. |
| `bivdi-agent` | The **agent host** (Phase 1, pre-kernel): constrained, time-limited, quota-bound agents. |
| `bivdi-event` | The **event bus**: typed, first-class events with correlation identity. |
| `bivdi-identity` | The **identity service**: identity kinds, petnames, selective disclosure. |
| `bivdi-runtime` | The **runtime node**: the six primitives composed into one running system. |
| `bivdi-wasm` | The **WASI runtime**: executes WASI modules with an explicit, least-authority capability context. |
| `bivdi-net` | The **networking model**: identity-based endpoints and flow capabilities (no raw sockets). |
| `bivdi-cli` | A CLI tying the pieces together for an end-to-end demo. |

## Provisional decisions (marked, not finalized)

- **Hash:** BLAKE3-256 is used as the content-address default. It is the leading proposal, **not** a decided item — see `docs/object-store-format.md`.
- **Unforgeability:** capabilities use opaque in-process ids. In Phase 0 (single-process Linux) this is a stand-in; true unforgeability is kernel-enforced in Bivdi Core.

## Build & test

```sh
cargo build --workspace
cargo test --workspace
```

## Run the demo

```sh
cargo run -p bivdi-cli                    # full six-primitive demo
cargo run -p bivdi-cli -- persist <path>  # object-store save/load round trip
cargo run -p bivdi-cli -- scenario        # RFC 0001 §3.4 agent exit-gate scenario
cargo run -p bivdi-cli -- wasm            # compile + run a WASI module
```

The `scenario` subcommand runs the Phase 1 agent-isolation exit gate end-to-end
through the composed runtime node: an agent with a leased Write capability can
write, a prompt-injection attempt to read an unrelated resource is denied, and
after the lease expires even the legitimate write is denied.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
