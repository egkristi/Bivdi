# Bivdi — Roadmap

**Status:** The **Bivdi Runtime is the product** (`D-003`). Phase 0 and the Runtime half of Phase 1 are implemented and runnable. The microkernel **Bivdi Core** is a parked research track; the kernel choice (`P-001`) is deferred indefinitely. This roadmap sequences the product (Milestones A–C) and records the parked track without letting it gate any deliverable.

**How to read this document:** each milestone lists its *goal*, *scope*, *deliverables*, *exit gate*, and *dependencies*. An exit gate is the objective, measurable condition that must be met before the next milestone begins — not an aspiration. Milestones are ordered by dependency and gated by outcomes, not by schedule.

**Process:** resolving an open question in `README.md` §16–17, or making any non-trivial design decision, is recorded as an RFC in `rfcs/` *before* work that depends on it begins.

> **A crate is not a milestone.** The Bivdi Runtime now has a crate for each of the six primitives, which is a real milestone — but a named crate is not the same as a satisfied deliverable, and an in-memory, single-process prototype is not the same as an exit gate. Work is complete when its exit condition is met, not when something exists under that name.

---

## Guiding principles for sequencing

1. **The Runtime is the product; the kernel is not.** The differentiator is the object/capability/state model applied to agent execution — not the kernel. The kernel is a possible future mechanism, not the product and not the critical path.
2. **Every milestone must ship value on its own.** A plan whose only payoff is at the final milestone dies before it gets there.
3. **The niche is the agent execution host** (`D-014`): the machine you run agents on, where what an agent touched is a queryable fact and what it could touch was bounded before it started.
4. **One contract, kernel-optional.** The WIT IDL (`D-015`) is the contract; the Runtime implements it today, and any future Core implements it later. No seL4 concept enters the IDL.
5. **Security is a gate, not a feature.** Each milestone has a security-relevant exit gate (hardening, provenance, the prompt-injection scenario).
6. **The kernel decision is deferred, not lost.** `P-001` stays open and RFC 0004 keeps the analysis; RFC 0002 §3.5 keeps the door open.

---

## Milestone A — Agent host on Linux (hardened)

**Goal.** Prove the core model is coherent and usable by a developer, on Linux, hardened — without a kernel.

**Scope.** The WIT IDL and conformance suite, the rights lattice corrected to a flag set, a durable object store, a queryable provenance interface, and seccomp + Landlock hardening of the runtime.

**Deliverables**

- `bivdi:core@0.1.0` in WIT (`D-015`) covering the six primitives, plus generated Rust bindings and a conformance suite.
- **Rights lattice fixed** — `Right` becomes a `flags right { read, write, execute, grant, signal, revoke }` with subset-inclusion attenuation, replacing the current three-value total order (RFC 0002 §5).
- **Object store** persisted to disk: content-addressed blobs, CAS cells, catalogs, snapshots, deterministic CBOR encoding.
- **Provenance query interface** — "what changed this, and what gave it the right to?" is a real query, not a `provenance_len()` counter.
- **Hardening** — seccomp filter and Landlock policy applied to the runtime process.
- A minimal developer SDK and CLI demonstrating the API end-to-end.

**Exit gate.** A developer can, on a Linux machine: write a program against the Bivdi API, hand it an attenuated capability, run it, and query provenance for everything it wrote — with no code running outside a sandbox.

### Current state against the exit gate

The gate has four clauses, mapped from the original Phase 0.

| Clause | Status | What is missing |
|---|---|---|
| *Write a program against the Bivdi API* | **Not met** | No IDL, no generated bindings, no SDK. Callers link Rust crates directly — a language calling convention, not the contract `D-005` requires. See [`rfcs/0002-interface-definition-language.md`](rfcs/0002-interface-definition-language.md). |
| *Hand it an attenuated capability* | **Met, in-process** | `bivdi-cap` mints, attenuates, leases and revokes, with subtree revocation. It does not survive a process boundary, and unforgeability is simulated with opaque ids. |
| *Query provenance for everything it wrote* | **Partly met** | Authority-relevant events are recorded. There is no query interface — `provenance_len()` is a counter, not an answer. |
| *With no code running outside a sandbox* | **Not met** | No seccomp filter and no Landlock policy anywhere in `runtime/`. |

### Current state against the deliverables

| Deliverable | Status |
|---|---|
| WIT IDL + conformance suite | **Not started** — the largest item in this milestone |
| Rights lattice (flag set) | **Not started** — prerequisite for the IDL |
| Object store (durable) | In-memory only: blobs, CAS cells, catalogs. **No on-disk encoding, no snapshots, no encryption** |
| Capability runtime | In-process: mint, attenuate, lease, revoke, provenance |
| State engine | Desired state, reconciliation, immutable generations, rollback |
| Event bus | In-memory: typed events, filters, correlation ids |
| Identity service | In-memory: kinds, fingerprints, petnames, attribute predicates |
| Agent host | Capability-constrained, leased, quota-bound agents; plan execution |
| Provenance query interface | **Not started** |
| Hardening (seccomp + Landlock) | **Not started** |
| Developer SDK and CLI | CLI demo only. **No SDK** |

**Remaining Milestone A work, in dependency order**

1. Fix the rights model — `flags` rather than an ordered enum (RFC 0002 §5). Cheapest now, prerequisite for the IDL.
2. Define the WIT IDL and conformance suite (RFC 0002). The suite *is* the "one contract" guarantee; without it that guarantee is an intention.
3. Persist the object store; give provenance a real query interface.
4. Harden the runtime with seccomp and Landlock — the fourth clause of the gate.

**Dependencies.** None (the IDL and conformance suite are already decided — `D-015`).

---

## Milestone B — WASI agent host

**Goal.** Run a real AI-agent task on the Runtime with delegated, time-limited capabilities, and prove a prompt-injection attempt gains nothing.

**Scope.** The wasmtime host (already present in `bivdi-wasm`), flow capabilities, and the RFC 0001 §3.4 scenario recorded as a repeatable run.

**Deliverables**

- A wasmtime host able to run real agent tooling as WASI components.
- **Flow capabilities** — an agent receives a capability naming a specific network endpoint with a trust anchor, never an unrestricted socket.
- The RFC 0001 §3.4 scenario, recorded end-to-end as a test and a demo.

**Exit gate.** The falsifiable scenario from [`rfcs/0001-target-niche.md`](rfcs/0001-target-niche.md) §3.4 passes as a recorded, repeatable run:

> An agent is granted a leased write capability to exactly one calendar entry and read access to exactly one document, with a 10-minute lease and a 20-action quota. The document contains an instruction directing the agent to forward the mailbox to an external address and delete the originals. On completion: no network flow capability was ever held, so no external connection is attempted or possible; no capability naming the mailbox exists in the agent's capability space; the provenance log shows every action attempted, the capability chain that authorised each permitted one, and an explicit denial for each attempt outside the grant; and the lease expires with remaining authority reaching zero without operator action.

**Security gate.** The runtime is hardened (Milestone A); the capability-derivation model is model-checked for authority leakage; the scenario run states which platform row it ran on (never overclaiming — the RFC 0003 principle applied to the Runtime's own claims).

**Dependencies.** Milestone A (IDL, rights lattice, provenance, hardening).

---

## Milestone C — Ship the product

**Goal.** An external operator runs the agent host and queries provenance without reading the source.

**Scope.** A container image, the CLI, an SDK, documentation, and an operator-facing provenance experience.

**Deliverables**

- A container image running the hardened Runtime (deployment, per `D-013`).
- A stable CLI (`bivdi-cli`) covering the operator workflow: grant, run, inspect provenance.
- An SDK with generated bindings from the WIT IDL.
- Documentation: getting-started, the capability model, the provenance model.
- Operator-facing provenance UX — the "what touched this" query that is half the value proposition.

**Exit gate.** An external operator runs the agent host from the container image and answers "what did this agent touch, and what authorised each touch?" from the provenance interface alone.

**Dependencies.** Milestone B.

---

## Parked: Bivdi Core (research track)

The microkernel Core is deferred research (`D-003`, `P-001` deferred). These items are **cut from the critical path** and recorded only so the option stays legible if the track is reactivated:

| Deferred | Notes |
|---|---|
| Microkernel bring-up | seL4 proposed; analysis in [`rfcs/0004-kernel-choice.md`](rfcs/0004-kernel-choice.md) |
| virtio drivers | `virtio-blk`, `virtio-net`, `virtio-console`, `virtio-rng` |
| Linux ABI layer | Starnix-pattern; plausibly larger than the rest of Core combined |
| Cloud NIC/storage drivers | AWS ENA/NVMe, GCP gVNIC/NVMe, Azure MANA/NetVSC/NVMe |
| Bare-metal reference hardware | Reference servers, driver VMs, micro-VM compatibility |
| Measured boot / sealing | vTPM on VM targets; TPM on bare metal |
| Desktop (optional) | Compositor, powerbox UI, Wayland proxy — already optional |

The platform-guarantees matrix (RFC 0003) and the kernel choice (RFC 0004) are both deferred alongside the track they serve. Their *principles* — never overclaim a guarantee, and no seL4 concept enters the IDL — remain in force for the Runtime.

---

## Cross-cutting work streams

These run across multiple milestones and are not tied to a single gate.

| Stream | Milestones | Notes |
|---|---|---|
| **Verification & assurance** | A → C | Fuzzing from day one; model-checking of capability derivation; never overclaim. |
| **Documentation & SDK** | A → C | The SDK needs excellent docs to be adopted; bindings generated from WIT. |
| **Governance** | A → C | Stage 1: technical lead + public RFC process. Stage 2 (post-1.0): elected technical steering committee; foundation holds trademark/domain. |
| **Naming** | A → B | Component naming decision. |

---

## Risks and their mitigations

| Risk | Severity | Mitigation |
|---|---|---|
| **Ecosystem / "nobody comes"** (Fuchsia is the cautionary tale) | Critical | Ship usable value each milestone; SDK from day one; agent-execution niche has a problem today rather than one to be argued for |
| **Scope explosion** | High | Runtime-first sequencing; the kernel is parked; explicit non-goals |
| **Prompt injection / compromised agent** | High | The Milestone B scenario makes the mitigation testable, not asserted |
| **Powerbox usability** | High | Deferred — the agent host is headless and operator-facing; a negative result on the desktop track changes the design, not the narrative |
| **Overclaiming assurance** | High | "Verification-oriented" until proven; published audits including unfixed findings |
| **seL4 licensing interaction** (seL4 is GPLv2-only) | Medium | Relevant only if Core is reactivated; open core (D-012) already isolates seL4 behind published interfaces |

---

## Effort and cost

Deliberately left out. Bivdi is sequenced by dependency and gated by outcomes; staffing and cost are planning questions to revisit once the first milestones prove the model, not numbers to commit to before any code exists.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
