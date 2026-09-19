# Bivdi — Specification v0.1

**Status:** Draft. This document states the *decided* model (see `README.md` §16 and `decisions.md`). Everything here is normative for the decided items; where a detail is **proposed** or **open**, it is marked as such and is *not* normative.

**Relationship:** `README.md` is the authoritative synthesis of goals and decisions; `ARCHITECTURE.md` elaborates the technical architecture. This specification is the next level down: the model that any implementation (Runtime or Core) must satisfy.

---

## 1. Scope and normative references

This specification defines the Bivdi **model** — the primitives, invariants, and semantics that are independent of the underlying substrate (Linux today, a microkernel tomorrow). It does *not* yet define the wire format, on-disk format, or concrete encodings; those are separate documents and are **open** (see §14).

Normative: `decisions.md` (D-001 … D-013). Non-normative background: `ARCHITECTURE.md`, `ROADMAP.md`.

---

## 2. The model

A Bivdi system is organized around six first-class primitives:

```text
IDENTITY + CAPABILITY + OBJECT + EVENT + DESIRED STATE + RESOURCE
```

These six — not the kernel — are the defining architecture. The kernel (or Linux runtime) is an *implementation* of this model, not the place where the model is invented.

### 2.1 Invariants (normative)

The following are load-bearing and never broken:

1. **No ambient authority.** A component's authority is exactly the capabilities it was handed. No `root`, no `sudo`, no global namespace; a process gains nothing from *who* launched it.
2. **Objects over paths.** Data lives in a typed, content-addressed object store; the filesystem is a compatibility *view*, never the conceptual center.
3. **Declarative state.** Configuration and workloads are described as desired state and reconciled; updates are immutable generations with transactional rollback.
4. **Events over polling.** Changes produce structured, first-class events.
5. **Compatibility without compromise.** Legacy software runs in isolated subsystems and never weakens the native model.
6. **AI without unrestricted authority.** Agents are capability-constrained, time-limited, quota-bound, and fully provenance-logged.
7. **Small trusted core.** Whatever must be correct to keep the system secure must be small enough to prove.
8. **IOMMU required.** Hardware without IOMMU is unsupported.
9. **Performance as a first-class attribute.** Performance is designed in, measured, and regression-gated — never won by weakening the security model.

---

## 3. Capability model (decided)

A **capability** is an unforgeable handle that both points to a resource and grants a specific right to it.

- **Unforgeable.** Forgery is *unrepresentable*, not merely hard.
- **Transferable.** A holder may pass a capability on, but only if it holds it.
- **Attenuable.** A holder may derive a strictly weaker capability (e.g., read-only, one subtree, one deadline). Attenuation never requires authority and is never reversible.
- **Revocable.** The granter may invalidate a capability and its entire derived subtree.
- **Scoped and delegatable.** Capabilities can be narrowed and handed onward; leases ("microphone for 5 minutes", "until this task ends") are the default shape of a grant, not an optional feature.

### 3.1 No root

There is no ambient superuser. The highest authority ("root capability") lives in hardware, is never granted to programs or agents, and is used only for recovery, key rotation, and ownership change.

### 3.2 Powerbox

Selecting an object in a trusted, system-drawn dialog *is* the grant. The application receives a capability to exactly the chosen object and nothing else.

### 3.3 Two tiers (proposed)

The architecture distinguishes two authority mechanisms; this distinction is **proposed**, pending the kernel decision (`P-001`):

- **Kernel capability** — local, live, dies with its process; authority *in use*.
- **Durable token** — transferable, offline-verifiable; authority *at rest and in transit*, bridged to kernel capabilities by a warden service.

See `token-format.md` (proposed) for the durable token.

### 3.4 Open questions

- Revocation of shared memory and of data already copied out of a component.
- The capability runtime's attenuation/revocation model at the Runtime level.

---

## 4. Object store (decided primitives)

The object store replaces the filesystem as the conceptual center. It is an unprivileged, restartable service with access only to the block devices it is handed.

Three primitives are decided:

| Primitive | Description |
|---|---|
| **Blob** | An immutable byte sequence named by its content hash. Identity *is* content. |
| **Cell** | A mutable single-slot holder of a blob reference, updated only by compare-and-swap. This is the *only* mutation primitive. |
| **Catalog** | A persistent map from names to blobs, cells, and sub-catalogs, reached **only by capability** — no root catalog, no `/`, no way to walk upward. |

Properties (decided):

- **Content addressing.** Blobs are named by hash; deduplication is inherent. *(Exact hash algorithm is **open**; BLAKE3 is a leading proposal, not a decision.)*
- **Deep immutability.** Writes are append-only; rollback is a pointer change.
- **Transactional.** Changes commit atomically; there is no partially-written state and no `fsck`.
- **Snapshots.** A retained root; O(1) to take, free to hold.
- **Per-object encryption** sealed to the boot measurement.
- **Crypto-shredding** (`D-010`): deletion destroys the key, so all copies and history become unreadable at once.
- **No machine-state persistence** (`D-007`): data and configuration are persisted; running instruction-level state is not.

### 4.1 Bounded scope

The object store does not replace PostgreSQL, Kafka, or large-scale object storage. Databases remain applications using the store's primitives.

See `object-store-format.md` for the model detail; the on-disk encoding is **open**.

---

## 5. State engine (decided)

The state engine accepts **desired state** and reconciles the system toward it:

```text
Desired State → Reconciler → Observed State → Actions → New Observed State
```

- Applies to workloads, networking, storage, permissions, devices, and configuration.
- **Generations** (`D-007`, `D-010`): an update creates a new immutable generation; activation is transactional and health-checked; failed generations roll back automatically.
- **Reconciliation vs. interaction:** reconciliation suits configuration and workloads; interactive, user-facing data uses direct transactional operations.

---

## 6. Event bus (decided)

Events are first-class typed records. All changes produce events on a single fabric consumed uniformly by applications, automation, synchronization, and observability.

- Typed, not text; defined in the IDL (§7).
- A shared correlation identity traces a single action across components.

---

## 7. Interfaces and the ABI (decided principles)

All inter-component communication uses **typed protocols** defined in a **language-neutral IDL** (`D-005`). Bindings are generated rather than hand-written.

- **The wire format is the contract**, not a language calling convention.
- A stable, language-neutral typed interface must support: objects, capabilities, typed calls, events, streams, cancellation, transactions, and structured errors.
- Large data crosses trust boundaries by **ownership transfer** (move semantics), not by shared mutable memory.
- **Open:** the specific IDL and its encoding. See `abi.md`.

---

## 8. Workloads (decided)

The execution unit is a **workload**: immutable code + an explicit capability set + a resource budget.

- **Budgets everywhere:** memory, CPU time, energy, latency class (realtime / interactive / batch / background), and I/O bandwidth. No unbounded queues.
- **Native application format:** WASI components (`D-004`) — first-class but not exclusive.
- Execution environments: WASI, native, Linux ABI (user-space translation), micro-VM.
- Isolation that containers provide on Linux is built in.
- See `manifest-schema.md` for the security boundary in human-readable form.

---

## 9. AI agents (decided)

Agents are first-class workloads but never receive unrestricted authority:

- **Attenuated delegation**, **time-limited**, **quota-bound**.
- **No escalation** — an agent can never delegate more than it holds.
- **Content is data, not instructions** — what an agent reads grants no new rights.
- **Irreversible actions require confirmation**, defined as policy.
- **Full provenance** — every write is traceable to the agent, the task, and the capability chain.
- **AI proposes, the OS enforces** — a natural-language request becomes an explicit, reviewable plan through the state engine and capability runtime.

---

## 10. Security and TCB (decided)

The trusted computing base is the minimum set whose failure breaks the model:

1. the CPU and its IOMMU;
2. the measured boot chain up to and including the kernel;
3. the kernel (choice **open**; seL4 proposed);
4. the root supervisor;
5. the root authority holding the seal/recovery key.

Everything else — the object store, drivers, network stack, compositor, applications — is outside the TCB. See `threat-model.md`.

---

## 11. Performance (decided)

Performance is a first-class attribute with regression gates. Reference targets live in `ARCHITECTURE.md` §20. The isolation cost is *budgeted* (e.g., within 30% of a monolithic kernel on syscall-heavy workloads), never traded for weakening the security model.

---

## 12. Two tracks (decided)

One model, two substrates, one interface contract:

| Track | Substrate | Purpose |
|---|---|---|
| **Bivdi Runtime** | Linux (hardened) | Prove the model, give a fast SDK |
| **Bivdi Core** | Microkernel (seL4 proposed), VM guest first | The product; small verified core is the value |

A program written against the shared IDL/API runs on either track unchanged.

---

## 13. Compatibility (decided)

| Level | Mechanism |
|---|---|
| Native | WASI components and native programs |
| Linux ABI | User-space syscall translation |
| Micro-VM | Isolated Linux/Windows kernel |
| Recompile | POSIX library (relibc-style) |

Each Linux process sees a filesystem view built from the capabilities it was handed; `/` is *its* root and nothing more.

---

## 14. Open questions (inputs to the next draft)

1. **IDL choice** and wire format (`abi.md` is blocked on this).
2. **Object-store mutation semantics** at the Runtime level (`object-store-format.md` is blocked on this).
3. **Capability-runtime attenuation/revocation** at the Runtime level.
4. **Hash algorithm** for content addressing (BLAKE3 proposed, not decided).
5. **Component naming** (`P-003`).
6. **Kernel choice** (`P-001`) — affects the durable-token/kernel-capability boundary.

Any resolution of these must be recorded as an RFC in `rfcs/` and reflected in `README.md` §16–17 and `decisions.md`.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
