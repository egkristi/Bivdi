# Bivdi — Architecture

**Status:** Concept / planning. This document is a technical sketch, not a frozen specification. Only the items marked *Decided* in `README.md` §16 are binding.

**Relation to other documents:** `README.md` is the authoritative synthesis of goals and decisions. This document elaborates the technical architecture. The `temp/` workgroup files are unedited background material.

> Component naming is **deferred**. This document uses generic, descriptive terms
> ("object store", "capability runtime", "state engine", …). No codenames or
> daemon-style abbreviations until a naming decision is recorded.

---

## 1. Architectural goals

The architecture exists to satisfy eight goals, in priority order:

1. **No ambient authority** — authority is exactly what is handed over, never inherited.
2. **Least authority by construction** — the default grant is nothing.
3. **Legible security** — a reviewer can read one manifest and state what a component can do.
4. **Recoverability** — every state change is versioned and reversible.
5. **Small trusted core** — the privileged kernel is tiny and provable.
6. **Compatibility without compromise** — legacy software is isolated, not privileged.
7. **Observability by design** — authority, state, and events are queryable.
8. **Portability** — the same model runs on Linux today and a microkernel tomorrow.

---

## 2. System overview

```text
┌────────────────────────────────────────────────────────────────────────┐
│  Applications                                                          │
│  native (WASI) · Linux ABI · micro-VM (legacy / Windows)               │
├────────────────────────────────────────────────────────────────────────┤
│  System services  (unprivileged · restartable · capability-confined)    │
│                                                                        │
│   object store   capability   state     event   identity  agent host   │
│                  runtime      engine    bus                         │
│   network stack  storage      provenance packaging compositor (later) │
│   component supervisor                                                 │
├────────────────────────────────────────────────────────────────────────┤
│  Capability boundary  (kernel capabilities ↔ durable tokens)           │
├────────────────────────────────────────────────────────────────────────┤
│  Drivers  (one isolation domain each · IOMMU-confined)                  │
│   virtio-* · nvme · ahci · xhci · net · gpu · rtc · tpm · hda          │
├────────────────────────────────────────────────────────────────────────┤
│  Microkernel  (proposed: seL4)                                          │
│   scheduling · memory · IPC · capabilities · interrupts · VMM          │
├────────────────────────────────────────────────────────────────────────┤
│  Hardware  (IOMMU required)                                             │
└────────────────────────────────────────────────────────────────────────┘
```

Authority flows strictly **downward** from a root supervisor and never upward. The kernel itself holds no policy: it answers "here is a capability, perform the operation it denotes", never "may this component do X".

---

## 3. The fundamental model

Where a conventional OS centers on `process + file + socket`, Bivdi centers on six primitives:

```text
IDENTITY + CAPABILITY + OBJECT + EVENT + DESIRED STATE + RESOURCE
```

These six — not the kernel — are the defining architecture. The kernel is an *implementation* of the model, and the same model can be implemented over a Linux runtime today and a microkernel tomorrow.

| Primitive | What it is | Analog in a traditional OS |
|---|---|---|
| **Identity** | Cryptographic identity for person, device, workload, service, object, organization | uid / hostname (but cryptographic, not ambient) |
| **Capability** | An unforgeable handle granting a specific right to a resource | file descriptor (but unforgeable, attenuable, revocable) |
| **Object** | Typed, content-addressed data with metadata, relationships, versions | file (but typed and versioned) |
| **Event** | A structured, first-class record of change | log line / inotify (but typed and queryable) |
| **Desired state** | Declarative specification of what should be | config file + init script (but reconciled) |
| **Resource** | CPU, memory, GPU, NPU, storage, network, device — with budgets | implicit kernel resources |

---

## 4. Capability model

### 4.1 Properties

A capability is an unforgeable handle that both **points to** a resource and **grants a right** to it. Forgery is *unrepresentable*, not merely cryptographically hard: a capability is an index validated by the kernel on every use, not a bit pattern the holder can inspect or manufacture.

- **Unforgeable.** Enforced by the kernel, not by convention.
- **Transferable.** A holder can pass a capability on, but only if it holds it.
- **Attenuable.** A holder can derive a strictly weaker capability (read-only, one subtree, one deadline). Attenuation never requires authority and is never reversible.
- **Revocable.** The granter can invalidate a capability and its entire derived subtree.

### 4.2 Two tiers of authority

Bivdi distinguishes two authority mechanisms; conflating them is a common design error.

| Tier | Scope | Persistence | Forgery |
|---|---|---|---|
| **Kernel capability** | Local, live, in a component's capability space | Dies with its process | Unrepresentable by construction |
| **Durable token** | Transferable, offline-verifiable | Persisted, transmitted | Cryptographically infeasible |

**Kernel capabilities** are authority *in use*. They cannot be written to disk or sent across a network. They are indices into a per-component capability space, with rights as a monotonically attenuating mask (`read`, `write`, `execute`, `grant`, `signal`, `revoke`). The only operation that ever changes a capability narrows it.

**Durable tokens** are authority *at rest and in transit*. They are bearer tokens in the macaroon tradition: an HMAC chain that makes attenuation verifiable offline. Any holder may append a restriction (caveat) and re-chain, producing a strictly weaker token without contacting an authority. Caveats can bind the token to an expiry, a component identity (by content hash), a required boot measurement, an invocation limit, a path prefix, a rate limit, or an audience.

A **warden** service bridges the two: it holds real kernel capabilities and, after verifying a presented token's chain and caveats, performs the operation with its own capability on the caller's behalf. Cryptography therefore stays strictly outside the trusted core.

### 4.3 Powerbox: the user gesture *is* the grant

Capability systems have historically foundered on usability. Bivdi uses the **powerbox** pattern: when a user selects a file in a system-drawn dialog, the selection *is* the grant. The application receives a capability to exactly the chosen object — and nothing else, not even the directory listing or the path. This reduces rather than increases the number of security dialogs, because the user is already expressing intent through a gesture.

### 4.4 Leases and revocation

- **Leases.** Permissions are leases, not permanent flags: "microphone for 5 minutes", "until this task finishes". Leases prevent permission hoarding.
- **Revocation** is layered because no single mechanism suffices: a forwarding intermediary that can be disconnected (revoking a subtree is detaching the intermediary); short default expiries; an epoch counter that invalidates all derived tokens at once; and an explicit revocation set.

Revocation of shared memory and of data already copied out of a component remains an **open problem**.

### 4.5 The root capability

"No root" does not mean no highest authority — it means the authority is not *ambient*. A **root capability** per machine and per identity:

- is held in hardware (TPM / secure key) and requires explicit local action,
- is never granted to programs or agents,
- is used only for recovery, key rotation, and ownership change.

---

## 5. The object store

The object store replaces the filesystem as the conceptual center. It is an unprivileged, restartable service with access only to the block devices it is handed.

### 5.1 Primitives

| Primitive | Description |
|---|---|
| **Blob** | An immutable byte sequence named by its content hash (e.g., BLAKE3). Identity *is* content, so deduplication and integrity verification are inherent. |
| **Cell** | A mutable single-slot holder of a blob reference, updated only by compare-and-swap. This is the *only* mutation primitive, so every lost-update race is a visible `cas` loop rather than a locking discipline. |
| **Catalog** | A persistent map from names to blobs, cells, and sub-catalogs. Reached **only by capability** — there is no root catalog, no `/`, and no way to walk upward. Two components holding different catalogs inhabit disjoint universes. |

### 5.2 Properties

- **Content addressing.** Blobs are named by hash; deduplication is automatic at the block level.
- **Deep immutability.** Writes are append-only; mutation creates a new node pointing at immutable parents. Rollback is a pointer change.
- **Transactions.** All changes commit atomically; there is no partially-written state and no `fsck`.
- **Snapshots.** A snapshot is a retained root — O(1) to take, free to hold.
- **Per-object encryption.** Each object (or group) is encrypted with its own key, sealed to the boot measurement. Objects can replicate to untrusted storage while leaking only their length.
- **Crypto-shredding.** Because history is immutable, deletion destroys the key, satisfying "right to be forgotten" without rewriting history.

### 5.3 Bounded scope

The object store offers transactions, snapshots, indexes, event streams, and replication. It does **not** replace PostgreSQL, Kafka, or large-scale object storage. Databases remain applications using the store's primitives.

---

## 6. State engine: desired state and reconciliation

The state engine accepts a declarative specification of desired state and reconciles the system toward it.

```text
Desired State → Reconciler → Observed State → Actions → New Observed State
```

- Applies to workloads, networking, storage, permissions, devices, and configuration.
- **Generations.** A system update is an immutable generation; activation is transactional and health-checked. A failed generation rolls back automatically.
- **Reconciliation vs. interaction.** Reconciliation suits configuration and workloads. Interactive, user-facing data uses direct transactional operations — the user expects immediate response, not eventual convergence.

---

## 7. The event bus

Events are first-class typed records. All changes — object lifecycle, workload lifecycle, capability grant/use/revocation, device attach/remove, policy change, generation activation/rollback — produce events on the bus.

- Applications, automation, synchronization, and observability consume the **same** fabric.
- Events are typed (defined in the IDL), not text that must be parsed.
- A shared correlation identity traces a single user action through applications, IPC, services, storage, and network.

---

## 8. Identity service

Identity is a system service, not something each application reinvents.

- **Identity kinds.** Person, device, workload, service, object, organization.
- **Cryptographic identity** underpins nodes and services.
- **Petnames.** Users assign local names to cryptographically identified parties; the system always displays the user's own name, never the remote party's self-claimed name (defeats phishing). Global names are used for discovery, never as a basis for trust.
- **Selective disclosure.** A service can verify "age ≥ 18" without receiving the birth date, via cryptographic proofs.

---

## 9. Agent host: AI as a constrained citizen

The agent host executes AI agents with **narrowly delegated, time-limited, quota-bound** capabilities.

- **Delegation is attenuated.** An agent gets write access to one calendar entry, not the whole calendar.
- **No escalation.** An agent can never delegate more than it holds.
- **Content is data, not instructions.** What an agent reads grants it no new authority (blocks prompt injection as a privilege-escalation vector).
- **Irreversible actions require confirmation**, defined as policy (send, delete, pay, publish).
- **Full provenance.** Every write is traceable to the agent, the task, and the capability chain.
- **AI proposes, the OS enforces.** A natural-language request becomes an explicit, reviewable plan that flows through the state engine and capability runtime like any other change.

---

## 10. IPC and interface definition

### 10.1 Typed protocols

All inter-component communication uses **typed protocols** defined in a language-neutral IDL (FIDL/Cap'n Proto style). Bindings are generated for Rust, C, C++, Go, Swift, and WASI components. Protocol violations are caught at compile time wherever possible.

> The **wire format is the contract**, not a language calling convention — which also solves Rust's lack of a stable ABI.

### 10.2 Async by default, with a fast sync path

Messages are asynchronous with promises. Synchronous calls exist for short, local operations where a fast IPC path gives low latency.

### 10.3 Large data without copying

Bulk transfer (graphics, ML, video) uses **ownership transfer**: a buffer is passed as a capability, and the sender loses access in the same instant — move semantics across trust boundaries. Shared, simultaneously mutable memory across trust boundaries is not allowed, except for explicitly agreed ring buffers between a driver and its service.

---

## 11. Scheduling and resources

### 11.1 Workloads, not processes

The execution unit is a **workload**: immutable code + explicit capability set + resource budget. Isolation that containers provide on Linux is built in.

### 11.2 Budgets everywhere

Every workload and request has explicit budgets: memory, CPU time, energy, latency class (realtime / interactive / batch / background), and I/O bandwidth. There are no unbounded queues; an exhausted budget degrades predictably rather than hanging the machine.

**CPU time is itself a capability.** A scheduling context `(budget, period, criticality)` is granted; admission control at bind time rejects oversubscription, so reservations are guaranteed rather than hoped for. A `donate` operation lets a client lend its reservation to a server for the duration of a call, accounting server work to the requester.

### 11.3 Heterogeneous silicon

CPU, GPU, NPU, and crypto accelerators are peers in the resource model. In practice, GPUs and NPUs carry their own (often closed) firmware and driver stacks; initially accelerators are exposed as capabilities from their driver domains, with a per-device budget.

---

## 12. Drivers and hardware

### 12.1 Strategy

1. **Native drivers** for standardized device classes (virtio, NVMe, AHCI, xHCI, a few NIC families) — small, well-documented, written in Rust where possible.
2. **Driver VMs** for everything else (GPU, Wi-Fi, other difficult hardware) — a deprivileged Linux VM behind IOMMU, exposed via virtio or dedicated protocols.
3. **Native drivers gradually take over**, one device class at a time.

A driver receives MMIO regions, an interrupt handler, an IOMMU domain restricted to its explicitly registered DMA buffers, and a scheduling context sized for its latency requirement. It receives **no** ability to touch memory outside its domain — which is why a compromised GPU driver is a graphics outage, not a root compromise. Every DMA buffer must be registered before use, and registration requires holding a capability to that memory.

### 12.2 Decided: VM-first driver target

Bivdi gets up and running **in virtual machines first**, targeting the most common VM engines: **KVM/QEMU, VirtualBox, VMware, Firecracker, Proxmox**, then the public clouds (AWS ENA/NVMe, GCP gVNIC/NVMe, Azure MANA/NVMe). `virtio` is the common denominator; the clouds add a small set of well-documented paravirtual NICs plus NVMe.

Hardware tiers: **(1)** virtualized (first target) → **(2)** reference servers → **(3)** reference laptop → **(4)** arbitrary hardware via driver VM → **never** hardware without IOMMU.

---

## 13. Networking

- **Identity-based.** Applications request a service by identity and capability, not IP address and port. The OS resolves identity to location and transport.
- **Flow capabilities, not raw sockets.** A component receives "you may connect to this endpoint with this trust anchor", never an unrestricted socket. There is no ambient namespace in which to `bind(0.0.0.0)`.
- **Encryption by default**, with mutual authentication, and no way to disable it for external connections.
- **DNS is a capability decision.** Name resolution and connection authority are one decision, closing DNS rebinding.
- **Uniform, not hidden.** The same API and capability model apply locally and remotely, but latency, timeouts, and partial failure are explicit in the types. The network is never disguised as local (Waldo et al., 1994).

---

## 14. Security boundaries and trusted core

### 14.1 The trusted computing base

The TCB is the minimum set whose failure breaks the security model:

1. the CPU and its IOMMU,
2. the measured boot chain up to and including the kernel,
3. the kernel itself,
4. the root supervisor,
5. the root authority holding the seal key.

**That is the complete list.** The object store, every driver, the network stack, the compositor, and every application are outside it. A defect in any of them is a bug, not a breach of the model.

### 14.2 The kernel

The kernel does only what requires the highest privilege: scheduling, virtual address spaces, IPC, capability enforcement, interrupt delivery, and virtualization support. It contains **no** device drivers, **no** filesystem, **no** network stack, and it **never parses a string**. Physical memory is distributed as untyped memory at boot; after boot the kernel allocates nothing, which removes an entire class of exhaustion and use-after-free defects.

**Status:** kernel choice is **open**; **seL4** is the leading proposal (formally verified for functional correctness down to machine code on supported architectures, plus integrity/confidentiality results). The alternative is a small original microkernel written following the seL4 methodology.

### 14.3 Hardware security

- **IOMMU required.** Every device can DMA only into buffers it was explicitly given.
- **IOMMU is necessary but not sufficient.** Thunderclap (2019) showed IOMMU protection can be bypassed when DMA buffers share pages. Bivdi uses per-buffer pages, strict (not deferred) IOTLB invalidation, and treats devices in one IOMMU group as a single security domain.
- **Measured boot** into a TPM or measured-boot log; storage sealed to the measurement, so a tampered system fails to unseal.
- **CHERI is optional** acceleration for fine-grained in-address-space isolation, not a design requirement.

---

## 15. Compatibility

| Level | Mechanism | Compatibility | Integration |
|---|---|---|---|
| **Native** | WASI components and native programs | New software | Full |
| **Linux ABI** | User-space syscall translation (Starnix pattern) | High for CLI/server software | Sees objects as files, bounded by capabilities |
| **Micro-VM** | Isolated Linux/Windows kernel | Very high | Via virtio and proxies |
| **Recompile** | POSIX library (relibc-style) | Source-available software | Medium |

- Each Linux process's filesystem view is a catalog it was handed; `/` is *its* root and nothing more. `open("/etc/passwd")` succeeds only if the granted catalog contains that entry.
- Windows programs run via a Wine-like environment inside the Linux layer or a micro-VM.
- **Cannibalization risk** (OS/2 effect) is acknowledged and mitigated: cheap porting to WASI, native advantages legacy software cannot get, and greenfield starting areas.

---

## 16. Two-track strategy

One interface contract, two substrates:

| Track | Substrate | Purpose |
|---|---|---|
| **Bivdi Runtime** | Linux (hardened with seccomp/Landlock) | Prove the object, capability, and state model quickly; SDK from day one |
| **Bivdi Core** | Microkernel (seL4 proposed), VM guest first, bare metal later | The product; the small verified core is the value proposition |

Programs written against the shared IDL/API run on either track without changes. The kernel is an implementation detail; the model is built first.

---

## 17. Boot chain

```text
[1] Platform firmware            — root of trust (UEFI SB / ARM TF-A / OpenSBI)
       │ measures into TPM / measured-boot log
[2] bivdi-shim (signed)          — minimal, verifies next stage
[3] bivdi-boot                   — loads kernel + root supervisor + composition
[4] Kernel                       — initialises, retypes memory, starts root supervisor
[5] Root supervisor              — instantiates the composition per manifest
[6] Steady state                 — storage key unsealed against expected measurements
```

Legitimate updates re-seal to the new expected measurement, so an update and a tamper are distinguishable. Remote attestation returns a signed quote plus the boot record, so a verifier can check *exactly which components are running, by content hash*.

---

## 18. Provenance and audit

An append-only, hash-chained log records every **authority-relevant** event: component instantiation with manifest digest; every capability mint, copy, move, and revoke with parent linkage; every token issuance, verification, and rejection; every object mutation with before/after hashes; every driver IOMMU-domain change; every update; every boot with its measurement set.

It records **authority, never content** — no data contents, keystrokes, or payloads, so the log is not itself a privacy liability. This distinction is load-bearing.

> "What changed my machine, and what gave it the right to?" is a query, not a forensic investigation.

---

## 19. Observability

Observability is built in, not bolted on: metrics per component and workload; structured logs as typed events; distributed tracing with causality across IPC and network; resource usage per budget and capability; security events; and a dependency graph between components. Programmable, verified hooks (eBPF-style) are available for deeper analysis, always as capabilities.

---

## 20. Recovery and migration

- **Recovery is declarative:** "restore system generation 42", "restore user data snapshot X".
- **Migration moves objects, identities, policies, workloads, and desired state** rather than requiring a machine to be rebuilt by hand.
- **No continuous machine-state persistence.** Bivdi persists data and configuration, not running instruction-level state. A reboot yields clean execution state over consistent data; services may checkpoint periodically.

---

## 21. Open architectural questions

Recorded honestly as inputs to spec v0.1:

1. **Revocation** of shared memory and copied-out data.
2. **Schema evolution** for persistent objects as types change.
3. **Large-data sharing** without shared mutable memory for GPU/ML/video.
4. **Cannibalization** — growing a native ecosystem alongside good compatibility.
5. **GUI in WASI** — wait, contribute, or define our own.
6. **Agent policy** — which actions always need human confirmation.
7. **Semantic indexing** without a broad-access indexer.
8. **Powerbox usability** — every capability system has foundered on the human interface.
9. **Driver availability** is the existential risk — narrow hardware list vs. a Linux driver shim.
10. **GPU acceleration vs. isolation** — there may be no way to have both.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
