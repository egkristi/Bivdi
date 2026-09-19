# Bivdi Runtime

The **Bivdi Runtime** is the product (`D-003`). It is the Linux-hosted implementation of the Bivdi model, hardened with seccomp/Landlock. The microkernel **Bivdi Core** is a parked research track, deferred indefinitely (`P-001`).

> **The kernel is not the product.** The product is the object, capability, and state model — concretely, the agent execution host (`D-014`). This runtime is the model in executable form.

## Status

The six primitives (object, capability, state, event, identity, agent) are
implemented with persistence, composed into an integrated node, and exercised by
a WASI runtime and CLI demo. **This is a userspace runtime on Linux, not an
operating system** — there is no kernel, no boot, and no VM guest, and none is
needed: the Runtime is the product. Remaining work is sequenced as Milestones
A–C in `ROADMAP.md`.

## Crates

| Crate | What it implements |
|---|---|
| `bivdi-object` | The **object store**: content-addressed blobs, CAS cells, catalogs, save/load. |
| `bivdi-cap` | The **capability runtime**: mint, attenuate, revoke (subtree), leases, provenance. |
| `bivdi-state` | The **state engine**: desired state, reconciliation, immutable generations. |
| `bivdi-agent` | The **agent host**: constrained, time-limited, quota-bound agents. |
| `bivdi-event` | The **event bus**: typed, first-class events with correlation identity. |
| `bivdi-identity` | The **identity service**: identity kinds, petnames, selective disclosure. |
| `bivdi-runtime` | The **runtime node**: the six primitives composed into one running system. |
| `bivdi-wasm` | The **WASI runtime**: executes WASI modules with an explicit, least-authority capability context. |
| `bivdi-net` | The **networking model**: identity-based endpoints and flow capabilities (no raw sockets). |
| `bivdi-sandbox` | The **sandbox**: a seccomp syscall allowlist and a read-only Landlock policy (best-effort; reports honestly when the host forbids it). |
| `bivdi-cli` | A CLI tying the pieces together for an end-to-end demo. |

The **conformance suite** (`tests/conformance`) encodes the "one contract" guarantee from RFC 0002 §8.5: the same test vectors must pass against the Runtime today and any future Core later.

The object store persists in two forms: provisional JSON (`save`/`load`) and **deterministic CBOR** (`save_cbor`/`load_cbor`, RFC 0002 §3.2).

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
