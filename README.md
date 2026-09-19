# Bivdi

**A capability-secure, object-centric, declarative operating system — designed from first principles.**

> *bivdit* (North Sámi) — to ask for, to request; also to hunt, to fish.
> In Bivdi, nothing has ambient authority. Everything must ask.

- **Project:** Bivdi
- **Domain:** [bivdi.com](https://bivdi.com)
- **Status:** Phase 0 and the Runtime half of Phase 1 are implemented and runnable (see [`runtime/`](runtime/)). The microkernel half of Phase 1 is blocked on `P-001`.
- **Document:** Project overview and decisions

---

## 1. What Bivdi is

Bivdi is a proposed operating-system architecture built around a single premise:

> **The computer should be a secure, distributed, stateful environment for people and workloads — not a pile of processes wrapped around a filesystem.**

It abandons several assumptions inherited from 1970s time-sharing systems:

- **No ambient authority.** A program has no access to anything it was not explicitly handed. There is no `root`, no `sudo`, and no user ID that grants power by virtue of who launched a program.
- **Objects over files.** Data lives in a typed, versioned, content-addressed object store. The filesystem survives as a compatibility view, not the conceptual center.
- **State over configuration.** The system is described declaratively and continuously reconciled toward desired state. Updates are immutable generations with transactional rollback.
- **Events over polling.** Changes produce structured, first-class events consumed uniformly by applications, automation, and observability.
- **Compatibility without compromise.** Existing Linux and Windows software runs in isolated subsystems that never weaken the native security model.
- **AI without unrestricted authority.** AI agents are first-class workloads, but an agent receives a narrow, time-limited, quota-bound capability set and everything it does is attributable.

Bivdi is explicitly **not** a Linux distribution, not a Kubernetes project, not a mobile OS, and not a blockchain.

---

## 2. Name

**Bivdi** is a North Sámi word derived from the verb *bivdit* ("to hunt, to fish; to ask for, to request"). The name fits the system: a hunter takes only what is needed, with the right tool — and in a capability system, everything must *ask*.

---

## 3. Core design principles

Ranked; when principles conflict, the higher one wins.

1. **No ambient authority.** All access is an explicitly granted capability.
2. **Least authority by construction.** The default grants nothing; every grant is a readable diff.
3. **Identity everywhere.** People, devices, workloads, services, objects, and organizations have identity.
4. **Objects over paths.** Typed, addressable objects replace the global namespace of the filesystem.
5. **State over configuration.** Declarative desired state, reconciled, versioned, and rollback-able.
6. **Events over polling.** Structured events are the native integration and observability mechanism.
7. **Declarative over imperative.** The system converges toward what is desired, not what a script last did.
8. **Immutable over mutable.** System generations and content-addressed data make rollback a pointer change.
9. **Small trusted core.** Whatever must be correct to keep the system secure must be small enough to prove.
10. **Compatibility without compromise.** Legacy software runs isolated; it never weakens the native model.
11. **AI foreslår, OS håndhever.** AI proposes; the OS enforces. No agent gets more than it was delegated.
12. **Portable, recoverable, observable by design.**
13. **Performance is a first-class attribute.** Performance is designed in, measured, and regression-gated — but never won by weakening the security model.
14. **Container-friendly, but containers are not a primitive.** The runtime runs in a container from the first executable, for deployment and compatibility. Containers are a compatibility view — they never become an architectural or security primitive.

---

## 4. The fundamental model

Traditional operating systems center on *processes, PIDs, UIDs, files, and sockets*. Bivdi centers on:

```text
IDENTITY + CAPABILITY + OBJECT + EVENT + DESIRED STATE + RESOURCE
```

Everything important has an identity. Authority is explicit. State is declared and reconciled. Changes produce structured events. Data and resources are addressable objects. These six primitives — not the kernel — are the defining architecture.

---

## 5. Architecture overview

```text
┌──────────────────────────────────────────────────────────────────┐
│  Applications                                                    │
│  native (WASI) │ Linux compatibility │ micro-VM (legacy/Windows) │
├──────────────────────────────────────────────────────────────────┤
│  System services (unprivileged, restartable, capability-confined) │
│  object store · capability runtime · state engine · event bus     │
│  identity · agent host · network · storage · provenance/audit     │
│  packaging/updates · component supervisor · compositor (later)    │
├──────────────────────────────────────────────────────────────────┤
│  Capability boundary                                             │
├──────────────────────────────────────────────────────────────────┤
│  Drivers (one isolated domain each, IOMMU-confined)               │
│  virtio-* · nvme · xhci · net · gpu · rtc · tpm …                │
├──────────────────────────────────────────────────────────────────┤
│  Microkernel (proposed: seL4)                                    │
│  scheduling · memory · IPC · capabilities · interrupts · VMM      │
├──────────────────────────────────────────────────────────────────┤
│  Hardware (IOMMU required)                                       │
└──────────────────────────────────────────────────────────────────┘
```

> **Component naming is deferred.** This README uses generic, descriptive terms
> (object store, capability runtime, state engine, event bus, identity service,
> agent host, …). Final names are an open decision.

### The kernel (open decision)

The long-term kernel minimizes the trusted computing base and does only what requires the highest privilege:

- scheduling (threads, with time as a budgeted resource)
- virtual address spaces / memory management
- inter-process communication
- capability enforcement
- interrupt delivery
- virtualization support (for driver VMs and micro-VMs)

Everything else — drivers, storage, networking, the window system — runs unprivileged in user space.

**Status:** kernel choice is *open*. The leading proposal is **seL4**, a formally verified capability microkernel (functional-correctness proofs down to machine code on supported architectures, plus integrity/confidentiality results). Reusing seL4 saves several person-years of verification. An alternative is a small original microkernel written following the seL4 methodology.

---

## 6. Security model

- **Capabilities.** Authority is an unforgeable handle that both points to an object and grants a specific right to it. Forgery is *unrepresentable*, not merely hard.
- **No ambient authority, no root.** A process gains no power from who launched it. There is no superuser to escalate to. The highest authority ("root capability") lives in hardware, is never granted to programs or agents, and is used only for recovery, key rotation, and ownership change.
- **Scoped, delegatable, attenuable, revocable.** Capabilities can be weakened (e.g., read-only, one subtree, one deadline), handed onward, and revoked — including revoking an entire derived subtree at once.
- **The user gesture *is* the grant (powerbox).** Selecting a file in a trusted system dialog *is* the capability assignment. Applications receive exactly the chosen object and nothing else — reducing, not increasing, security dialogs.
- **Leases.** Permissions are leases, not permanent flags: "microphone for 5 minutes", "until this task is done". This prevents permission hoarding.
- **Provenance.** Every write records who wrote it, with which capability chain, from which code (identified by content hash). "What changed my machine, and what gave it the right to?" is a query, not a forensic investigation.
- **Hardware boundaries.** IOMMU is required. Every device can DMA only into buffers it was explicitly given. Measured boot into a TPM/measured-boot log; storage sealed to the boot measurement.
- **Threat model (summary).** Defends against malicious applications, compromised dependencies and drivers, DMA attackers, network attackers, prompt-injection attacks on agents, supply-chain attacks, ransomware, and lost devices. Out of scope for v1: malicious silicon, novel side channels, physical attack on a running machine, traffic analysis.

---

## 7. Storage and state

- **Typed object store.** Data lives in a typed, schema-validated, content-addressed graph with metadata, relationships, and version history. The hierarchical filesystem remains as *one* view of the graph.
- **Deep immutability.** Writes are append-only; mutation creates a new revision pointing to immutable parents. Rollback to any prior state is a pointer change.
- **Transactional, crash-consistent.** All changes are transactions; there is no partially-written state and no `fsck`.
- **Crypto-shredding for deletion.** Because history is immutable, deletion is implemented by destroying the per-object encryption key, satisfying "right to be forgotten" without rewriting history.
- **No continuous machine-state persistence.** Bivdi persists *data and configuration*, not running instruction-level machine state. A reboot yields clean execution state over consistent data. Services may take periodic checkpoints.
- **Bounded scope.** The object store offers transactions, snapshots, indexes, event streams, and replication. It does *not* replace PostgreSQL, Kafka, or large-scale object storage — databases remain applications using OS primitives.
- **Flies survive as contracts.** Bivdi does not kill the file. Objects have a canonical serialized form and can always be exported/imported as files; legacy programs see objects as files via a virtual filesystem view.

---

## 8. Declarative state and reconciliation

Users, applications, and administrators describe *desired state*. The system continuously reconciles observed state toward it:

```text
Desired State → Reconciler → Observed State → Actions → New Observed State
```

- Applies to workloads, networking, storage, permissions, devices, and configuration.
- **Immutable system generations.** An update creates a new immutable generation instead of mutating the running system. Activation is transactional and health-checked; failed generations roll back automatically.
- Reconciliation suits configuration and workloads; *interactive* data uses direct, transactional operations.

---

## 9. Workloads, sandboxing, and execution

The fundamental execution unit is a **workload**: immutable code, an explicit capability set, and a resource budget. Isolation that containers provide on Linux is built in.

Execution environments:

- **WebAssembly (WASI)** — the first-class native application target: portable, capability-oriented, strongly isolated.
- **Native** — for maximum performance, hardware access, and systems software.
- **Linux compatibility** — unmodified Linux binaries via a user-space syscall layer.
- **Micro-VM** — a real Linux/Windows kernel in an isolated VM for heavyweight legacy workloads.

**Budgets everywhere.** Memory, CPU time, energy, latency class, and I/O bandwidth are explicit per workload. There are no unbounded queues; exhausted budgets degrade predictably rather than hanging the machine.

---

## 10. AI agents as constrained citizens

The clearest differentiator from existing operating systems:

- **Attenuated delegation.** An agent gets, for example, write access to one calendar entry and read access to one email thread — not the whole calendar and inbox.
- **Time-limited, quota-bound.** Capabilities expire when the task ends or a deadline passes; CPU, memory, network calls, cost, and number of actions are capped.
- **No escalation.** An agent can never delegate more than it holds.
- **Content is data, not instructions.** What an agent reads grants it no new rights (blocks prompt-injection as a privilege-escalation vector).
- **Irreversible actions require confirmation.** Sending, deleting, paying, and publishing can require explicit human approval, defined as policy.
- **Full provenance.** Every write is traceable to the agent, the task, and the capability chain.
- **AI proposes, the OS enforces.** A natural-language request becomes an explicit, reviewable plan that flows through the same state engine and capability runtime as any other change.

Agents are first-class, but never receive unrestricted authority.

---

## 11. Networking and distribution

- **Identity-based networking.** Applications request a service by identity and capability rather than hard-coding IP addresses and ports. Encryption and mutual authentication are default.
- **No raw sockets as ambient authority.** A component receives *flow capabilities* ("you may connect to this endpoint with this trust anchor"), never an unrestricted socket.
- **Uniform, but not hidden.** The same API and capability model apply locally and remotely — but latency, timeouts, and partial failure are explicit in the types. The network is never disguised as local.
- **Offline-first.** Devices are replicas and caches; disconnected operation is normal. CRDTs are used where meaningful, with explicit conflict resolution where they are not.
- **Clusters as explicit units.** Multiple Bivdi machines can form an explicitly configured cluster for workload and object placement; never an illusion of one giant machine.

---

## 12. Compatibility

| Level | Mechanism | Notes |
|---|---|---|
| **Native** | WASI components and native Bivdi programs | Full capabilities, objects, events |
| **Linux ABI** | User-space syscall translation (Starnix pattern) | High for CLI/server software; each process sees a virtual filesystem built from its capabilities |
| **Micro-VM** | Isolated Linux/Windows kernel | Very high compatibility; lower integration |
| **Recompile** | POSIX library (relibc-style) | For software with source |

- Windows programs run via a Wine-like translation environment inside the Linux layer or a micro-VM.
- The first browser on Bivdi will run in the Linux layer or a micro-VM; a native browser is a multi-year project.
- **Cannibalization risk is acknowledged:** a too-good compatibility layer could prevent a native ecosystem from forming. Bivdi's answers are cheap porting (WASI as the main path), native advantages legacy software cannot get (capabilities, provenance, generations, cross-device sharing), and starting in greenfield areas.

---

## 13. Drivers and hardware strategy

Driver support is historically where new operating systems die. Bivdi's approach is layered:

1. **Native drivers** for standardized device classes (virtio, NVMe, AHCI, xHCI, a few NIC families) — small, well-documented, written in Rust where possible.
2. **Driver VMs** for everything else (GPU, Wi-Fi, and other difficult hardware) — a deprivileged Linux VM behind IOMMU, exposed via virtio or dedicated protocols.
3. **Native drivers gradually take over** one device class at a time.

### Decided: start with the most common VM engines

From a driver-support perspective, Bivdi will get up and running **in virtual machines first**, targeting the most common VM engines and hypervisors:

- **KVM / QEMU** (`virtio-blk`, `virtio-net`, `virtio-gpu`, `virtio-console`, `virtio-rng`)
- **VirtualBox**
- **VMware**
- **Firecracker**
- **Proxmox**
- **Public clouds:** AWS (ENA/NVMe), Google Cloud (gVNIC/NVMe), Azure (MANA/NetVSC/NVMe)

`virtio` is the common denominator across KVM, VirtualBox, VMware, Firecracker, and Proxmox; the major public clouds add a small set of well-documented paravirtual NIC drivers plus NVMe. VM-first means a short, well-understood driver list, instant availability of a development target, and none of the hardest desktop-hardware problems (GPU, Wi-Fi, suspend).

Hardware support in tiers:

| Tier | Scope | Status |
|---|---|---|
| **1: Virtualized** | KVM, VirtualBox, VMware, Firecracker, Proxmox, AWS, GCP, Azure | First target |
| **2: Reference servers** | One or two chosen x86-64 / ARM server platforms | Second target |
| **3: Reference laptop** | One chosen laptop (e.g., Framework) | Later phase |
| **4: Arbitrary hardware** | Via driver VM where IOMMU exists | Best effort |
| **—: No IOMMU** | — | Never supported |

---

## 14. Strategy: two parallel tracks

Bivdi develops along two tracks sharing one interface contract (IDL) and one API, so programs written against one run on the other without changes:

| Track | Substrate | Purpose |
|---|---|---|
| **Bivdi Runtime** | Linux (hardened with seccomp/Landlock) | Prove the object model, state engine, and APIs quickly; give developers an SDK from day one |
| **Bivdi Core** | Microkernel (seL4 proposed), as a VM guest first, bare metal later | The product. The small verified trusted core is the value proposition and must be present from the start |

> **The kernel is not the product.** The product is the object, capability, and state model. The kernel is the mechanism that enforces it. The model is built first; the kernel is an implementation of the model, not the place where the model is invented.

---

## 15. Roadmap

Phases below synthesize the workgroup proposals. They are ordered by dependency and gated by outcomes, not by schedule — no timeline is estimated here. See `ROADMAP.md` for full deliverables, exit gates, and dependencies.

| Phase | Content | Success criterion |
|---|---|---|
| **0 — Foundation** | Spec v0.1, threat model, IDL, Bivdi Runtime on Linux (object store, capability runtime, state engine) | A developer writes a program, grants an attenuated capability, sees provenance for everything it writes |
| **1 — Agent host in a VM** | Bivdi Core on microkernel with virtio drivers, WASI runtime, agent host, provenance; runs on KVM/Firecracker | An agent runs a real task with delegated, time-limited capabilities; prompt injection gains nothing beyond the delegation |
| **2 — Cloud & pilots** | ENA/gVNIC/MANA + NVMe drivers, attestation, cluster, Linux compatibility layer | ≥1 external org runs production workloads in a public cloud |
| **3 — Bare metal** | Reference servers, driver VMs, micro-VM compatibility, more native drivers | Bivdi runs on its own hardware with the same guarantees |
| **4 — Desktop (optional)** | Reference laptop, graphical surface, Wayland proxy for legacy apps | Depends on funding |

**Feasibility note.** The verified kernel is the *easy* part (seL4 already exists). The hard part is everything around it — ecosystem, compatibility, drivers, adoption. Fuchsia is the cautionary control experiment.

---

## 16. Decisions

### Decided

| ID | Decision |
|---|---|
| D-001 | Name is **Bivdi**; domain is **bivdi.com** |
| D-002 | First-class driver target is the **most common VM engines** (KVM, VirtualBox, VMware, Firecracker, Proxmox, then AWS/GCP/Azure) |
| D-003 | Two parallel tracks: **Bivdi Runtime** (Linux) and **Bivdi Core** (microkernel), sharing one IDL/API |
| D-004 | **WASI components** as the native application format |
| D-005 | **Rust** for services and drivers; language-neutral IDL for protocols |
| D-006 | **IOMMU required**; hardware without IOMMU is unsupported |
| D-007 | Files survive as export format and contracts; **no persistence of running machine state** |
| D-008 | Distribution is uniform, but network failure/latency is never hidden |
| D-009 | GPU and Wi-Fi run in deprivileged **driver VMs** |
| D-010 | **Crypto-shredding** for deletion in versioned storage |
| D-011 | **Performance is a first-class attribute** — designed in, measured, and gated; never won by weakening the security model |
| D-012 | **Open-core licensing**: freely implementable spec; permissive SDKs (MIT OR Apache-2.0); MPL-2.0 for Bivdi's own core/services/drivers; proprietary commercial/enterprise layer; DCO (no CLA); interface exception. See [`LICENSING.md`](LICENSING.md). |
| D-013 | **Container-friendly, not container-primitive**: the runtime runs in a container from the first executable; containers are a deployment/compatibility concern, never an architectural or security primitive. |

### Proposed (not yet final)

| ID | Proposal |
|---|---|
| P-001 | **seL4** as the microkernel (alternative: original microkernel, seL4 methodology) |
| P-002 | First target market / niche |
| P-003 | Component naming scheme |
| ~~P-004~~ | ~~License model~~ → **resolved** as D-012 (open core). See [`LICENSING.md`](LICENSING.md). |
| P-005 | Governance structure and RFC process |

---

## 17. Open questions

Recorded honestly — these are not solved in the design and are inputs to spec v0.1:

1. **Revocation.** How to revoke access to shared memory and to data already copied out of a component?
2. **Schema evolution.** How to migrate persistent objects safely when their types change?
3. **Efficient sharing of large data.** How far does ownership transfer cover GPU/ML/video workloads without shared mutable memory?
4. **Cannibalization.** How to ensure a native ecosystem grows when compatibility layers are good?
5. **GUI in WASI.** Wait for a standard, contribute to one, or define our own?
6. **Agent policy.** Which actions always require human confirmation, and how is that expressed?
7. **Semantic indexing.** Useful cross-data search without a broad-access indexer?
8. **Powerbox usability.** Every capability system has foundered on the human interface; needs user testing.
9. **Driver availability is the existential risk.** Hold the line on a narrow hardware list, or accept a Linux driver shim with its assurance cost?
10. **GPU acceleration vs. isolation.** There may be no way to have both hardware-accelerated graphics and a small TCB.

---

## 18. Repository structure (proposed)

```
bivdi/
├── docs/                 # specs, threat model, ABI, attribution
├── kernel/               # the trusted core (microkernel)
├── runtime/              # Bivdi Runtime on Linux
├── services/             # object store, capability runtime, state engine, event bus,
│                         # identity, agent host, network, storage, provenance/audit,
│                         # packaging, compositor (later)
├── drivers/              # one directory per driver, one isolated component each
├── lib/                  # capability-typed stdlib, syscall bindings, C ABI
├── compat/               # Linux ABI layer, micro-VM integration
├── tools/                # build, package, audit, image tooling
├── tests/                # property, fault-injection, conformance, fuzz
└── rfcs/                 # all design decisions with rationale
```

---

## 19. License

**Open core** (`D-012`). See [`LICENSING.md`](LICENSING.md) for the full model. Summary:

- **Specification:** freely implementable.
- **SDKs, libraries, ABI headers:** MIT OR Apache-2.0 (dual).
- **Bivdi's own core/services/drivers:** MPL-2.0.
- **Third-party code (e.g., seL4):** its own license (seL4 is GPLv2-only).
- **Documentation:** CC BY 4.0.
- **Commercial/enterprise layer:** proprietary, sold separately.

> seL4 (GPLv2-only) is never part of Bivdi's licensed core and is never resold; it is used as a separate component behind published interfaces.

---

## 20. Further reading

- [seL4](https://sel4.systems) · [WASI](https://wasi.dev) · [Fuchsia](https://fuchsia.dev) · [Genode](https://genode.org) · [Redox](https://www.redox-os.org) · [Qubes OS](https://www.qubes-os.org) · [NixOS](https://nixos.org) · [Firecracker](https://firecracker-microvm.github.io)
- Klein et al., *seL4: Formal Verification of an OS Kernel* (SOSP 2009)
- LeVasseur et al., *Unmodified Device Driver Reuse … via Virtual Machines* (OSDI 2004)
- Waldo et al., *A Note on Distributed Computing* (1994)
- Hardy, *The Confused Deputy* (1988)

---

*Bivdi — nothing has ambient authority. Everything must ask.*
