# Bivdi — Specification

**Status:** Normative for the *decided* items. Where a detail is **proposed** or **open**, it is marked as such and is **not** normative.

**Role of this document.** [`README.md`](README.md) says *what Bivdi is*; this document says *what a Bivdi system MUST do*. It is the normative protocol/semantic specification that any implementation — the Bivdi Runtime on Linux today, or a Bivdi Core microkernel tomorrow — MUST satisfy. Implementation mechanics live in [`ARCHITECTURE.md`](ARCHITECTURE.md); the sequenced plan lives in [`ROADMAP.md`](ROADMAP.md).

**Conformance.** A system is a *conforming Bivdi system* if and only if it satisfies the requirements expressed with **MUST**/**MUST NOT** in this document. Requirements expressed with **SHOULD**/**SHOULD NOT** are recommended and MAY be ignored only for an explicit, documented reason. Requirements expressed with **MAY** are optional. **Decided** items are listed in [`docs/decisions.md`](docs/decisions.md), the single source of truth for the decided/proposed boundary.

**Normative language.** The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** in this document are to be interpreted as described in RFC 2119.

---

## 1. Scope

This specification defines the Bivdi **model** — the primitives, invariants, and semantics that are independent of the underlying substrate (Linux today, a microkernel tomorrow). It does **not** yet define the wire format, the on-disk format, or concrete encodings; those are separate documents and are **open** (see §15).

Normative references: [`docs/decisions.md`](docs/decisions.md) (D-001 … D-015). Non-normative background: [`ARCHITECTURE.md`](ARCHITECTURE.md), [`ROADMAP.md`](ROADMAP.md), and the elaboration documents under [`docs/`](docs/README.md).

---

## 2. The model

A Bivdi system is organized around six first-class primitives:

```text
IDENTITY + CAPABILITY + OBJECT + EVENT + DESIRED STATE + RESOURCE
```

These six — not the kernel — are the defining architecture. The kernel (or the Linux runtime) is an *implementation* of this model, not the place where the model is invented.

| Primitive | What it is |
|---|---|
| **Identity** | Cryptographic identity for person, device, workload, service, object, and organization. |
| **Capability** | An unforgeable handle granting a specific right to a resource. |
| **Object** | Typed, content-addressed data with metadata, relationships, and version history. |
| **Event** | A structured, first-class record of change. |
| **Desired state** | A declarative specification of what should be. |
| **Resource** | CPU, memory, GPU, NPU, storage, network, device — with explicit budgets. |

---

## 3. Core invariants

The following are load-bearing. A conforming Bivdi system MUST satisfy every one of them, and any design, code, or proposal that contradicts one is wrong.

1. **No ambient authority.** A component's authority MUST be exactly the capabilities it was handed. There MUST NOT be `root`, `sudo`, or a global namespace; a process MUST NOT gain anything from *who* launched it.
2. **Objects over paths.** Data MUST live in a typed, content-addressed object store; the filesystem MUST be a compatibility *view*, never the conceptual center.
3. **Declarative state.** Configuration and workloads MUST be described as desired state and reconciled; updates MUST be immutable generations with transactional rollback.
4. **Events over polling.** Changes MUST produce structured, first-class events.
5. **Compatibility without compromise.** Legacy software MUST run in isolated subsystems and MUST NOT weaken the native model.
6. **AI without unrestricted authority.** Agents MUST be capability-constrained, time-limited, quota-bound, and fully provenance-logged.
7. **Small trusted core.** Whatever must be correct to keep the system secure MUST be small enough to prove.
8. **Performance as a first-class attribute.** Performance MUST be designed in, measured, and regression-gated — and MUST NOT be won by weakening the security model.

> **IOMMU required (`D-006`) is parked** alongside the deferred Core track: it governs driver isolation on bare metal/Core. The Runtime's DMA isolation is owned by the Linux host kernel. See [`docs/decisions.md`](docs/decisions.md) and [`rfcs/0003-platform-guarantees.md`](rfcs/0003-platform-guarantees.md).

---

## 4. Capability model

A **capability** is an unforgeable handle that both points to a resource and grants a specific right to it. A conforming system MUST enforce all of the following properties:

- **Unforgeable.** Forgery MUST be *unrepresentable*, not merely cryptographically hard. In a kernel implementation this is an index validated by the kernel on every use, not a bit pattern the holder can inspect or manufacture. In the Runtime (in-process) implementation it is currently simulated with opaque ids; see §14.
- **Transferable.** A holder MAY pass a capability on, but only if it holds it.
- **Attenuable.** A holder MAY derive a strictly weaker capability (read-only, one subtree, one deadline). Attenuation MUST NOT require authority and MUST be irreversible.
- **Revocable.** The granter MUST be able to invalidate a capability and its entire derived subtree.
- **Scoped and delegatable.** Capabilities MUST be narrowable and handable onward. Leases ("microphone for 5 minutes", "until this task ends") are the default shape of a grant.

### 4.1 No root

There MUST NOT be an ambient superuser. The highest authority ("root capability") MUST live in hardware, MUST NOT be granted to programs or agents, and MUST be used only for recovery, key rotation, and ownership change.

### 4.2 Powerbox

Selecting an object in a trusted, system-drawn dialog *is* the grant. The application MUST receive a capability to exactly the chosen object and nothing else — not the directory listing, not the path.

### 4.3 Two tiers of authority (proposed)

The architecture distinguishes two authority mechanisms; this distinction is **proposed**, pending the kernel decision (`P-001`), and is therefore **not** normative:

- **Kernel capability** — local, live, dies with its process; authority *in use*.
- **Durable token** — transferable, offline-verifiable; authority *at rest and in transit*, bridged to kernel capabilities by a warden service.

See [`docs/token-format.md`](docs/token-format.md) (proposed).

---

## 5. Object store

The object store replaces the filesystem as the conceptual center. It MUST be an unprivileged, restartable service with access only to the block devices it is handed.

Three primitives are decided and MUST be present:

| Primitive | Description |
|---|---|
| **Blob** | An immutable byte sequence named by its content hash. Identity *is* content. |
| **Cell** | A mutable single-slot holder of a blob reference, updated only by compare-and-swap. This is the **only** mutation primitive. |
| **Catalog** | A persistent map from names to blobs, cells, and sub-catalogs, reached **only by capability** — there MUST NOT be a root catalog, a `/`, or any way to walk upward. |

Properties (decided):

- **Content addressing.** Blobs MUST be named by hash; deduplication is inherent. *(The exact hash algorithm is **open**; BLAKE3 is a leading proposal, not a decision.)*
- **Deep immutability.** Writes MUST be append-only; mutation MUST create a new revision pointing to immutable parents. Rollback MUST be a pointer change.
- **Transactional.** Changes MUST commit atomically; there MUST NOT be partially-written state. *(The Runtime's current persistence is a whole-graph serialiser, not yet a crash-consistent store — see [`ROADMAP.md`](ROADMAP.md).)*
- **Snapshots.** A snapshot MUST be a retained root — O(1) to take, free to hold.
- **Per-object encryption** sealed to the boot measurement.
- **Crypto-shredding** (`D-010`): deletion MUST destroy the key, so all copies and history become unreadable at once. *(A mechanism, not a claim to satisfy any legal standard.)*
- **No machine-state persistence** (`D-007`): data and configuration MUST be persisted; running instruction-level state MUST NOT be.

### 5.1 Bounded scope

The object store MUST NOT replace PostgreSQL, Kafka, or large-scale object storage. Databases remain applications using the store's primitives.

See [`docs/object-store-format.md`](docs/object-store-format.md) for the model detail; the on-disk encoding is **open**.

---

## 6. State engine

The state engine accepts **desired state** and reconciles the system toward it:

```text
Desired State → Reconciler → Observed State → Actions → New Observed State
```

- Reconciliation MUST apply to workloads, networking, storage, permissions, devices, and configuration.
- **Generations** (`D-007`): an update MUST create a new immutable generation; activation MUST be transactional and health-checked; a failed generation MUST roll back automatically.
- **Reconciliation vs. interaction:** reconciliation suits configuration and workloads; interactive, user-facing data MUST use direct transactional operations.

---

## 7. Event bus

Events MUST be first-class typed records. All changes MUST produce events on a single fabric consumed uniformly by applications, automation, synchronization, and observability.

- Events MUST be typed, not text; types MUST be defined in the interface definition language (§8).
- A shared correlation identity MUST trace a single action across components.

---

## 8. Interfaces and the ABI

All inter-component communication MUST use **typed protocols** defined in a **language-neutral interface definition language** (`D-005`). Bindings MUST be generated rather than hand-written.

- **The wire format is the contract**, not a language calling convention.
- A stable, language-neutral typed interface MUST support: objects, capabilities, typed calls, events, streams, cancellation, transactions, and structured errors.
- Large data MUST cross trust boundaries by **ownership transfer** (move semantics), not by shared mutable memory.
- **Decided** (`D-015`): **WIT** is the interface definition language; the Component Model canonical ABI is used in-process, deterministic CBOR across boundaries; bindings are generated. **No seL4 concept may enter the interface definition language.** See [`docs/abi.md`](docs/abi.md) and [`rfcs/0002-interface-definition-language.md`](rfcs/0002-interface-definition-language.md).

---

## 9. Workloads and execution

The execution unit MUST be a **workload**: immutable code + an explicit capability set + a resource budget.

- **Budgets everywhere:** memory, CPU time, energy, latency class (realtime / interactive / batch / background), and I/O bandwidth MUST be explicit per workload. There MUST NOT be unbounded queues; an exhausted budget MUST degrade predictably rather than hang the machine.
- **Native application format:** WASI components (`D-004`) — first-class but not exclusive.
- Execution environments: WASI, native, Linux ABI (user-space translation), micro-VM.
- Isolation that containers provide on Linux MUST be built in.
- See [`docs/manifest-schema.md`](docs/manifest-schema.md) for the security boundary in human-readable form.

---

## 10. AI agents

Agents are first-class workloads but MUST NOT receive unrestricted authority:

- **Attenuated delegation** — an agent receives the narrowest set sufficient for its task.
- **Time-limited and quota-bound** — capabilities MUST expire when the task ends or a deadline passes; CPU, memory, network calls, cost, and number of actions MUST be capped.
- **No escalation** — an agent MUST NOT delegate more than it holds.
- **Content is data, not instructions** — what an agent reads MUST grant no new rights (this blocks prompt injection as a privilege-escalation vector).
- **Irreversible actions require confirmation** — sending, deleting, paying, and publishing MUST be definable as policy requiring explicit human approval.
- **Full provenance** — every write MUST be traceable to the agent, the task, and the capability chain.
- **AI proposes, the OS enforces** — a natural-language request MUST become an explicit, reviewable plan that flows through the state engine and capability runtime like any other change.

See [`docs/ai-agents.md`](docs/ai-agents.md).

---

## 11. Networking and distribution

- **Identity-based.** Applications MUST request a service by identity and capability rather than hard-coding IP addresses and ports. Encryption and mutual authentication MUST be the default.
- **Flow capabilities, not raw sockets.** A component MUST receive *flow capabilities* ("you may connect to this endpoint with this trust anchor"), never an unrestricted socket. There MUST NOT be an ambient namespace in which to `bind(0.0.0.0)`.
- **Name resolution is a capability decision.** Resolution and connection authority MUST be one decision, closing DNS rebinding.
- **Uniform, but not hidden.** The same API and capability model MUST apply locally and remotely — but latency, timeouts, and partial failure MUST be explicit in the types. The network MUST NOT be disguised as local.
- **Offline-first.** Devices MUST be replicas and caches; disconnected operation MUST be normal. CRDTs are used where meaningful, with explicit conflict resolution where they are not.
- **Clusters as explicit units.** Multiple machines MAY form an explicitly configured cluster; it MUST NOT be presented as one giant machine.

See [`docs/networking.md`](docs/networking.md).

---

## 12. Compatibility

| Level | Mechanism | Notes |
|---|---|---|
| **Native** | WASI components and native Bivdi programs | Full capabilities, objects, events |
| **Linux ABI** | User-space syscall translation (Starnix pattern) | Each process sees a filesystem view built from its capabilities |
| **Micro-VM** | Isolated Linux/Windows kernel | Very high compatibility; lower integration |
| **Recompile** | POSIX library (relibc-style) | For software with source |

- Each Linux process's filesystem view MUST be a catalog it was handed; `/` is *its* root and nothing more.
- Windows programs MUST run via a Wine-like translation environment inside the Linux layer or a micro-VM.
- The first browser MUST run in the Linux layer or a micro-VM; a native browser is a multi-year project.
- **Cannibalization risk** (a too-good compatibility layer preventing a native ecosystem) MUST be acknowledged and mitigated: cheap porting (WASI as the main path), native advantages legacy software cannot get, and greenfield starting areas.

See [`docs/compatibility.md`](docs/compatibility.md).

---

## 13. Security and trusted computing base

The trusted computing base (TCB) MUST be the minimum set whose failure breaks the model:

1. the CPU and its IOMMU;
2. the measured boot chain up to and including the kernel;
3. the kernel (choice **open**; seL4 proposed);
4. the root supervisor;
5. the root authority holding the seal/recovery key.

**That is the complete list.** The object store, every driver, the network stack, the compositor, and every application MUST be outside the TCB. A defect in any of them is a bug, not a breach of the model.

- **Authority is the only thing logged.** The provenance log MUST record *authority*, never content — no data contents, keystrokes, or payloads. The log MUST NOT itself be a privacy liability.
- **Measured boot** MUST seal storage to a TPM/measured-boot measurement; a tampered system MUST fail to unseal.
- **IOMMU** (where present) MUST confine each device to the DMA buffers it was explicitly given.

See [`docs/threat-model.md`](docs/threat-model.md), [`docs/provenance.md`](docs/provenance.md), [`docs/boot-attestation.md`](docs/boot-attestation.md).

---

## 14. Runtime as the product

The **Bivdi Runtime** (Linux, hardened) is the product; the microkernel **Bivdi Core** is a parked research track, deferred indefinitely (`P-001`). One interface contract (the WIT interface definition language, `D-015`) MUST be retained so a program written against it runs on any future Core unchanged.

| Track | Substrate | Status |
|---|---|---|
| **Bivdi Runtime** | Linux (hardened with seccomp/Landlock) | **The product** |
| **Bivdi Core** | Microkernel (seL4 proposed), VM guest first, bare metal later | Parked research; not on the critical path |

The Runtime's in-process capability unforgeability is a **simulated stand-in** (opaque ids) for kernel enforcement; content addressing uses **BLAKE3-256** as a **proposed, not decided** algorithm. These provisional choices MUST remain marked as such and MUST NOT be promoted to decided without an RFC. See [`ROADMAP.md`](ROADMAP.md).

---

## 15. Open questions (inputs to the next draft)

Recorded honestly — these are not solved in the design and are inputs to a future spec revision:

1. **Revocation.** Authority revocation is implemented (subtree-wide); *information* revocation — data already copied out of a component — is impossible in general. Shared-memory revocation remains open.
2. **Schema evolution.** How to migrate persistent objects safely when their types change?
3. **Efficient sharing of large data.** How far does ownership transfer cover GPU/ML/video workloads without shared mutable memory?
4. **Cannibalization.** How to ensure a native ecosystem grows when compatibility layers are good?
5. **GUI in WASI.** Wait for a standard, contribute to one, or define our own?
6. **Agent policy.** Which actions always require human confirmation, and how is that expressed?
7. **Semantic indexing.** Useful cross-data search without a broad-access indexer?
8. **Powerbox usability.** Every capability system has foundered on the human interface; needs user testing.
9. **Driver availability is the existential risk** — relevant to the Core research track, not the Runtime product.
10. **GPU acceleration vs. isolation.** There may be no way to have both hardware-accelerated graphics and a small TCB.
11. **Hash algorithm** for content addressing (BLAKE3 proposed, not decided).
12. **Component naming** (`P-003`).
13. **Kernel choice** (`P-001`) — affects the durable-token/kernel-capability boundary.

Any resolution of these MUST be recorded as an RFC in [`rfcs/`](rfcs/README.md) and reflected in [`docs/decisions.md`](docs/decisions.md) and this document.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
