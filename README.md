# Bivdi

**The machine you run agents on — where what an agent touched is a queryable fact, and what it could touch was bounded before it started.**

> *bivdit* (North Sámi) — to ask for, to request; also to hunt, to fish.
> In Bivdi, nothing has ambient authority. Everything must ask.

- **Project:** Bivdi
- **Domain:** [bivdi.com](https://bivdi.com)
- **Status:** Phase 0 and the Runtime half of Phase 1 are implemented and runnable (see [`runtime/`](runtime/)). The **Bivdi Runtime is the product**; the microkernel Core is a parked research track.
- **Document:** Project overview and decisions

---

## 1. What Bivdi is

Bivdi is an operating system for running AI agents, built around a single premise:

> **The computer should be a secure, distributed, stateful environment for people and workloads — not a pile of processes wrapped around a filesystem.**

The product is the **agent execution host**: the machine you run agents on, where what an agent touched is a queryable fact and what it could touch was bounded before it started. It abandons several assumptions inherited from 1970s time-sharing systems:

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
│  Microkernel (deferred research track: seL4 proposed)             │
│  scheduling · memory · IPC · capabilities · interrupts · VMM      │
├──────────────────────────────────────────────────────────────────┤
│  Hardware (IOMMU required)                                       │
└──────────────────────────────────────────────────────────────────┘
```

> The diagram shows the full eventual shape. **Today the Runtime is the product** and runs the services layer directly on Linux (hardened with seccomp/Landlock), with the driver and microkernel layers deferred to the Core research track.

> **Component naming is deferred.** This README uses generic, descriptive terms
> (object store, capability runtime, state engine, event bus, identity service,
> agent host, …). Final names are an open decision.

### The kernel (deferred)

The Runtime runs on the Linux kernel today, hardened with seccomp/Landlock. The long-term vision of a microkernel — the **Bivdi Core** track — is parked research, not the product. When and if it is revisited, it would minimize the trusted computing base and do only what requires the highest privilege:

- scheduling (threads, with time as a budgeted resource)
- virtual address spaces / memory management
- inter-process communication
- capability enforcement
- interrupt delivery
- virtualization support (for driver VMs and micro-VMs)

Everything else — drivers, storage, networking, the window system — runs unprivileged in user space.

**Status:** kernel choice is *deferred indefinitely* (`P-001`). The leading proposal remains **seL4**, a formally verified capability microkernel (functional-correctness proofs; on AArch64 also integrity and confidentiality, with binary-level verification only on AArch32 — see [`rfcs/0004-kernel-choice.md`](rfcs/0004-kernel-choice.md)). Until it is decided, the IDL forbids seL4 concepts (`D-015`), keeping the Core option open without letting the kernel's vocabulary leak into Bivdi's contracts.

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
- **Transactional, crash-consistent.** The object model is designed so that ordinary crash recovery does not require filesystem-style repair; all changes are transactions in the model. *(The Runtime's current persistence is a whole-graph serialiser, not yet a crash-consistent store — see `ROADMAP.md`.)*
- **Crypto-shredding for deletion.** Because history is immutable, deletion is implemented by destroying the per-object encryption key — Bivdi-managed encrypted replicas can be rendered unreadable through key destruction. *(A mechanism, not a claim to satisfy any legal standard; not yet implemented in the Runtime.)*
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

## 14. Strategy: the Runtime is the product

The **Bivdi Runtime** (Linux, hardened with seccomp/Landlock) is the shipped product. It implements the object, capability, and state model now, reaches the agent-execution-host niche, and gives developers an SDK from day one.

The **Bivdi Core** (microkernel) is a parked research track, deferred indefinitely. One shared interface contract (the WIT IDL, `D-015`) is retained so that any future Core can run the same programs unchanged — but Core is not a parallel product track, and no current deliverable depends on it.

> **The kernel is not the product.** The product is the object, capability, and state model — concretely, the agent execution host. The kernel is a possible future mechanism that enforces the model; it is not where the model is invented, and it is not on the critical path.

---

## 15. Roadmap

Milestones below are ordered by dependency and gated by outcomes, not by schedule — no timeline is estimated here. See `ROADMAP.md` for full deliverables, exit gates, and dependencies. The Runtime is the product; the microkernel Core is parked research.

| Milestone | Content | Success criterion |
|---|---|---|
| **A — Agent host on Linux** | WIT IDL + conformance suite, rights lattice (flag set), durable object store, queryable provenance, seccomp + Landlock hardening | An agent runs a real task on the hardened Runtime; provenance for what it touched is queryable |
| **B — WASI agent host** | wasmtime host, flow capabilities, the RFC 0001 §3.4 scenario as a recorded, repeatable run | A prompt-injection attempt yields no access beyond the delegation, verified by the recorded run |
| **C — Ship the product** | container image, CLI, SDK, docs, provenance UX for operators | An external operator runs the agent host and queries provenance without reading the source |
| **Core (parked)** | Microkernel bring-up (seL4 proposed), virtio drivers | Reactivated only if/when the research track is resumed; no current deliverable depends on it |

**Feasibility note.** The verified kernel was always the *easy* part (seL4 already exists); the hard part is everything around it — ecosystem, compatibility, drivers, adoption. The Runtime-first strategy does the hard part first and treats the kernel as optional.

---

## 16. Decisions

### Decided

| ID | Decision |
|---|---|
| D-001 | Name is **Bivdi**; domain is **bivdi.com** |
| D-002 | ~~First-class driver target is the **most common VM engines** (KVM, VirtualBox, VMware, Firecracker, Proxmox, then AWS/GCP/Azure)~~ → **Parked** alongside Core (P8) |
| D-003 | **Bivdi Runtime is the product; Bivdi Core is a parked research track.** One IDL/API is retained so a future Core can run the same programs |
| D-004 | **WASI components** as the native application format |
| D-005 | **Rust** for services and drivers; language-neutral IDL for protocols |
| D-006 | ~~**IOMMU required**; hardware without IOMMU is unsupported~~ → **Parked** alongside Core (P8) |
| D-007 | Files survive as export format and contracts; **no persistence of running machine state** |
| D-008 | Distribution is uniform, but network failure/latency is never hidden |
| D-009 | GPU and Wi-Fi run in deprivileged **driver VMs** |
| D-010 | **Crypto-shredding** for deletion in versioned storage |
| D-011 | **Performance is a first-class attribute** — designed in, measured, and gated; never won by weakening the security model |
| D-012 | **Open-core licensing**: freely implementable spec; permissive SDKs (MIT OR Apache-2.0); MPL-2.0 for Bivdi's own core/services/drivers; proprietary commercial/enterprise layer; DCO (no CLA); interface exception. See [`LICENSING.md`](LICENSING.md). |
| D-013 | **Container-friendly, not container-primitive**: the runtime runs in a container from the first executable; containers are a deployment/compatibility concern, never an architectural or security primitive. |
| D-014 | **First target niche is the agent execution host** — the machine you run agents on, where what an agent touched is a queryable fact and what it could touch was bounded before it started. See [`rfcs/0001-target-niche.md`](rfcs/0001-target-niche.md). |
| D-015 | **WIT as the IDL**; Component Model canonical ABI in-process, deterministic CBOR across boundaries; generated bindings. **No seL4 concept enters the IDL.** See [`rfcs/0002-interface-definition-language.md`](rfcs/0002-interface-definition-language.md). |

### Proposed (not yet final)

| ID | Proposal |
|---|---|
| P-001 | **seL4** as the microkernel (alternative: original microkernel, seL4 methodology) — **deferred indefinitely**; Core is parked research |
| P-003 | Component naming scheme |
| ~~P-002~~ | ~~First target market / niche~~ → **resolved** as D-014. See [`rfcs/0001-target-niche.md`](rfcs/0001-target-niche.md). |
| ~~P-004~~ | ~~License model~~ → **resolved** as D-012 (open core). See [`LICENSING.md`](LICENSING.md). |
| P-005 | Governance structure and RFC process |

---

## 17. Open questions

Recorded honestly — these are not solved in the design and are inputs to spec v0.1:

1. **Revocation.** Authority revocation is implemented (subtree-wide); *information* revocation — data already copied out of a component — is impossible in general. The distinction is stated in `docs/capabilities.md` §7; shared-memory revocation remains open.
2. **Schema evolution.** How to migrate persistent objects safely when their types change?
3. **Efficient sharing of large data.** How far does ownership transfer cover GPU/ML/video workloads without shared mutable memory?
4. **Cannibalization.** How to ensure a native ecosystem grows when compatibility layers are good?
5. **GUI in WASI.** Wait for a standard, contribute to one, or define our own?
6. **Agent policy.** Which actions always require human confirmation, and how is that expressed?
7. **Semantic indexing.** Useful cross-data search without a broad-access indexer?
8. **Powerbox usability.** Every capability system has foundered on the human interface; needs user testing.
9. **Driver availability is the existential risk** — relevant to the Core research track, not the Runtime product. Hold the line on a narrow hardware list, or accept a Linux driver shim with its assurance cost?
10. **GPU acceleration vs. isolation.** There may be no way to have both hardware-accelerated graphics and a small TCB.

---

## 18. Repository structure

```
bivdi/
├── docs/                 # specs, threat model, ABI, attribution
├── runtime/              # Bivdi Runtime on Linux — the product
├── rfcs/                 # all design decisions with rationale
│
│   # Deferred research track (Bivdi Core) — created when/if reactivated:
├── kernel/               # the trusted core (microkernel)
├── drivers/              # one directory per driver, one isolated component each
├── lib/                  # capability-typed stdlib, syscall bindings, C ABI
├── compat/               # Linux ABI layer, micro-VM integration
├── services/             # native service components (Core)
├── tools/                # build, package, audit, image tooling
└── tests/                # property, fault-injection, conformance, fuzz
```

The Runtime lives entirely under `runtime/`. The `kernel/`, `drivers/`, `lib/`, `compat/`, `services/`, and `tests/` directories belong to the deferred Core research track and are created only if that track is reactivated.

---

## 19. License

**Open core** (`D-012`). See [`LICENSING.md`](LICENSING.md) for the full model. Summary:

- **Specification:** freely implementable.
- **SDKs, libraries, ABI headers:** MIT OR Apache-2.0 (dual).
- **Bivdi's own core/services/drivers:** MPL-2.0.
- **Third-party code (e.g., seL4):** its own license (seL4 is GPLv2-only).
- **Documentation:** CC BY 4.0.
- **Commercial/enterprise layer:** proprietary, sold separately.

> seL4 (GPLv2-only) is never part of Bivdi's licensed core and its source is never conveyed as part of Bivdi's MPL-2.0 code; it is used as a separate component behind published interfaces. (GPLv2 permits sale; the constraint Bivdi respects is source conveyance, not resale.)

---

## 20. Further reading

- [Getting started](docs/getting-started.md) — build, test, and run the Runtime in minutes.
- [seL4](https://sel4.systems) · [WASI](https://wasi.dev) · [Fuchsia](https://fuchsia.dev) · [Genode](https://genode.org) · [Redox](https://www.redox-os.org) · [Qubes OS](https://www.qubes-os.org) · [NixOS](https://nixos.org) · [Firecracker](https://firecracker-microvm.github.io)
- Klein et al., *seL4: Formal Verification of an OS Kernel* (SOSP 2009)
- LeVasseur et al., *Unmodified Device Driver Reuse … via Virtual Machines* (OSDI 2004)
- Waldo et al., *A Note on Distributed Computing* (1994)
- Hardy, *The Confused Deputy* (1988)

---

*Bivdi — nothing has ambient authority. Everything must ask.*
