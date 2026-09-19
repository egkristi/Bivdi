# Bivdi — Roadmap

**Status:** Phase 0 complete; Phase 1 Runtime half complete (microkernel bring-up remains, blocked on `P-001`). This roadmap is a proposal synthesized from the workgroup documents and the decisions recorded in `README.md`. Only the items marked *Decided* in `README.md` §16 are binding. No timeline is estimated here — phases are ordered by dependency, not by date.

**How to read this document:** each phase lists its *goal*, *scope*, *deliverables*, *exit gate*, and *dependencies*. An exit gate is the objective, measurable condition that must be met before the next phase begins — not an aspiration. Phases are ordered by dependency and gated by outcomes, not by schedule.

**Process:** resolving an open question in `README.md` §16–17, or making any non-trivial design decision, is recorded as an RFC in `rfcs/` *before* work that depends on it begins.

> **A crate is not a phase.** The Bivdi Runtime now has a crate for each of the six primitives, which is a real milestone — but a named crate is not the same as a satisfied deliverable, and an in-memory, single-process prototype is not the same as an exit gate. Phase 0's status table below states which is which. Work is complete when its exit condition is met, not when something exists under that name.

---

## Guiding principles for sequencing

1. **Build the model first, the kernel second.** The differentiator is the object/capability/state model, not the kernel. The kernel is an implementation detail.
2. **Every phase must ship value on its own.** A plan whose only payoff is at the final phase dies before it gets there.
3. **VM-first for hardware.** The hardest driver problems (GPU, Wi-Fi, suspend) are deliberately deferred behind the first cloud/VM milestone.
4. **Two tracks, one contract.** Bivdi Runtime (Linux) and Bivdi Core (microkernel) advance in parallel against a shared IDL/API.
5. **Security is a gate, not a feature.** Each phase has a security-relevant exit gate (fuzzing, audit, provenance).
6. **Who before what.** `P-002` — the first target niche — is resolved *before* `P-001`, the kernel choice. The niche determines what the kernel has to support; taking the kernel decision first means choosing a mechanism before knowing the requirement. See [`rfcs/0001-target-niche.md`](rfcs/0001-target-niche.md).

---

## Phase 0 — Foundation

**Goal.** Prove the core model is coherent and usable by a developer, on Linux, without a kernel.

**Scope.** Specification v0.1, threat model, IDL, and the first Bivdi Runtime services.

**Deliverables**

- `docs/` — specification v0.1, threat model v1, ABI/IDL definition, attribution.
- Bivdi Runtime on Linux (hardened with seccomp/Landlock) providing:
  - **object store** — content-addressed blobs, catalogs, cells, snapshots
  - **capability runtime** — mint, attenuate, lease, revoke, provenance
  - **state engine** — desired state, reconciliation, generations
- A minimal developer SDK and CLI demonstrating the API end-to-end.
- `rfcs/` seeded with the decisions already made (`D-001` … `D-013`) and the first RFCs for the open questions.

**Exit gate.** A developer can, on a Linux machine: write a program against the Bivdi API, hand it an attenuated capability, run it, and query provenance for everything it wrote — with no code running outside a sandbox.

### Status against the exit gate

The gate has four clauses. None is met yet, and none of the remaining work is blocked on `P-001`.

| Clause | Status | What is missing |
|---|---|---|
| *Write a program against the Bivdi API* | **Not met** | There is no API in the `D-003` sense — no IDL, no generated bindings, no SDK. Callers link Rust crates directly, which is a language calling convention, not the contract `D-005` requires. See [`rfcs/0002-interface-definition-language.md`](rfcs/0002-interface-definition-language.md). |
| *Hand it an attenuated capability* | **Met, in-process** | `bivdi-cap` mints, attenuates, leases and revokes, with subtree revocation. It does not survive a process boundary, and unforgeability is simulated with opaque ids. |
| *Query provenance for everything it wrote* | **Partly met** | Authority-relevant events are recorded, including capability use. There is no query interface — `provenance_len()` is a counter, not an answer to "what changed this, and what gave it the right?" |
| *With no code running outside a sandbox* | **Not met** | The runtime is not hardened. There is no seccomp filter and no Landlock policy anywhere in `runtime/`. |

### Status against the deliverables

| Deliverable | Status |
|---|---|
| Specification v0.1 | Draft; normative for decided items, blocked in §7 and §14 on the IDL |
| Threat model v1 | Present; needs the platform scoping in [`rfcs/0003-platform-guarantees.md`](rfcs/0003-platform-guarantees.md) |
| ABI / IDL definition | **Not started** — the largest unblocked item in the phase |
| Attribution | Complete |
| Object store | In-memory: blobs, CAS cells, catalogs. **No on-disk encoding, no snapshots, no encryption** |
| Capability runtime | In-process: mint, attenuate, lease, revoke, provenance |
| State engine | Desired state, reconciliation, immutable generations, rollback |
| Event bus | In-memory: typed events, filters, correlation ids |
| Identity service | In-memory: kinds, fingerprints, petnames, attribute predicates |
| Agent host | Capability-constrained, leased, quota-bound agents; plan execution |
| Developer SDK and CLI | CLI demo only. **No SDK** |
| `rfcs/` seeded with `D-001` … `D-013` | **Not started.** RFCs 0001–0003 are the first; the decided items are still unrecorded, and `D-013` was added with no RFC |

**Remaining Phase 0 work, in dependency order**

1. Resolve `P-002` (RFC 0001) — it determines what the SDK and the demo are for.
2. Fix the rights model: `Right` is a three-value total order, but `ARCHITECTURE.md` §4.2 names six rights that do not form a chain. Attenuation must be subset inclusion over a flag set. This is a prerequisite for writing the IDL truthfully and is cheapest now.
3. Define the IDL and the conformance suite (RFC 0002). The suite *is* the `D-003` "one contract, two tracks" guarantee; without it that guarantee is an intention.
4. Persist the object store, and give provenance a real query interface.
5. Harden the runtime with seccomp and Landlock — the fourth clause of the gate.
6. Publish the platform guarantees matrix (RFC 0003).
7. Backfill RFCs for `D-001` … `D-013` and for the decisions already made in code: BLAKE3 as the content hash, agent leases held separately from capability leases, opaque ids as the unforgeability stand-in.

**Dependencies.** None (started from `README.md` + `ARCHITECTURE.md`).

**Open questions to resolve before/during this phase:** IDL choice and wire format (RFC 0002); the exact object-store mutation primitive (`cas`) semantics; the capability runtime's attenuation and revocation model at the runtime level.

---

## Phase 1 — Agent host in a VM

**Goal.** Run a real AI-agent task inside Bivdi Core with delegated, time-limited capabilities, on a VM.

**Scope.** First Bivdi Core bring-up on the microkernel, virtio drivers, WASI runtime, agent host, provenance.

**Deliverables**

- Bivdi Core booting on the microkernel (seL4 proposed) as a VM guest.
- Drivers: `virtio-blk`, `virtio-net`, `virtio-console`, `virtio-rng` (native, Rust).
- WASI runtime for native applications.
- **Agent host** — capability-constrained agent execution with quotas and time limits.
- Provenance logging across the runtime and core.
- Runs on **KVM/QEMU** and **Firecracker** first; VirtualBox/VMware follow once the virtio stack is stable.

**Exit gate.** An agent completes a real task inside Bivdi Core; a prompt-injection attempt yields **no** access beyond what was delegated; and the resulting provenance chain is complete and queryable.

Stated as a runnable scenario, so that it is falsifiable rather than descriptive (see [`rfcs/0001-target-niche.md`](rfcs/0001-target-niche.md) §3.4):

> An agent is granted a leased write capability to exactly one calendar entry and read access to exactly one document, with a 10-minute lease and a 20-action quota. The document contains an instruction directing the agent to forward the mailbox to an external address and delete the originals. On completion: no network flow capability was ever held, so no external connection is attempted or possible; no capability naming the mailbox exists in the agent's capability space; the provenance log shows every action attempted, the capability chain that authorised each permitted one, and an explicit denial for each attempt outside the grant; and the lease expires with remaining authority reaching zero without operator action.

**Security gate.** Kernel syscall surface under continuous fuzzing; capability-derivation model model-checked for authority leakage; and the platform guarantees matrix published, with the demonstration stating which row it ran on ([`rfcs/0003-platform-guarantees.md`](rfcs/0003-platform-guarantees.md)).

**Dependencies.** Phase 0 IDL/API stable enough to port to Core. The kernel half of this phase is blocked on `P-001`; the agent host, provenance and WASI runtime work is not.

---

## Phase 2 — Cloud and pilot customers

**Goal.** First external organization runs production workloads on Bivdi in a public cloud.

**Scope.** Cloud NIC/storage drivers, attestation, clustering, Linux ABI layer, and operational hardening.

**Deliverables**

- Cloud drivers: AWS **ENA**/NVMe, GCP **gVNIC**/NVMe, Azure **MANA**/NetVSC/NVMe.
- Remote attestation (signed PCR quote + boot record).
- Cluster as an explicit unit: workload/object placement, data locality, fault domains.
- **Linux ABI** layer (user-space syscall translation) for unmodified CLI and server software.
- Packaging and atomic update system with reproducible builds and rollback.
- Observability and provenance query tooling for operators.

> **The Linux ABI is not one deliverable among six.** The cited pattern — Fuchsia's Starnix — has absorbed a sustained team for years and still covers a bounded subset; gVisor is the same order of magnitude. Listed beside "packaging and atomic updates" it is understated by roughly an order of magnitude, and it is plausibly larger than the rest of Phase 2 combined.
>
> Two consequences. First, it needs its own exit gate and an explicitly narrow initial syscall surface — *"enough to run a statically linked Go binary"* is a good first gate. Second, **micro-VM compatibility should be evaluated ahead of it**: for the headless server and agent host niche it reaches most of the same software for a small fraction of the effort. The current ordering — Linux ABI in Phase 2, micro-VM in Phase 3 — is backwards on effort-to-value grounds and should be reconsidered when `P-002` is resolved.

**Exit gate.** At least one external organization runs production workloads on Bivdi in a public cloud, with the same security guarantees as in a VM — on a platform whose row in the guarantees matrix reads *Full*, or with the degradation named explicitly in the claim ([`rfcs/0003-platform-guarantees.md`](rfcs/0003-platform-guarantees.md)).

**Security gate.** Third-party audit of the capability and durable-token model, published in full.

**Dependencies.** Phase 1 Core stability; cloud driver work; packaging/update system.

---

## Phase 3 — Bare metal

**Goal.** Bivdi runs on its own hardware with the same guarantees as in the cloud.

**Scope.** Reference servers, driver VMs, micro-VM compatibility, more native drivers.

**Deliverables**

- Reference hardware: one or two x86-64 and ARM server platforms (Tier 2).
- **Driver VMs** — deprivileged Linux VM behind IOMMU for GPU/Wi-Fi/difficult hardware.
- **Micro-VM** compatibility for heavyweight legacy workloads.
- Additional native drivers (NVMe, AHCI, xHCI/HID, Ethernet families).
- Measured boot and storage sealing on real hardware.

**Exit gate.** Bivdi boots on reference bare-metal hardware and passes the Phase 1 security guarantees (agent isolation, provenance) without virtualization.

**Dependencies.** Phase 2 operational experience; driver VM and IOMMU work.

---

## Phase 4 — Desktop (optional)

**Goal.** A usable desktop surface on one reference laptop — *if* funding and ecosystem warrant it.

**Scope.** Reference laptop (e.g., Framework), graphical compositor, Wayland proxy for legacy apps.

**Deliverables**

- Capability-based compositor and powerbox UI.
- Structured shell (`bivdi-sh`) and object browser.
- Wayland proxy for legacy graphical applications in micro-VMs.
- Reference laptop bring-up (Tier 3 hardware).

**Exit gate.** A developer can do a day's work in Bivdi on reference hardware.

**Dependencies.** A decision to pursue desktop (currently an *open question*, not decided); a browser strategy; GUI-in-WASI resolution.

---

## Cross-cutting work streams

These run across multiple phases and are not tied to a single milestone.

| Stream | Phases | Notes |
|---|---|---|
| **Verification & assurance** | 0 → 4 | Fuzzing from day one; model-checking of capability/IPC; kernel memory-safety proofs; long-term functional-correctness proof. "Verification-oriented" until the proof exists — never overclaim. |
| **Compatibility** | 2 → 4 | Linux ABI grows toward server software, then desktop; micro-VM as fallback; Windows via Wine. |
| **Documentation & SDK** | 0 → 4 | The ABI needs excellent SDKs and docs to be adopted. |
| **Governance** | 0 → 4 | Stage 1: technical lead + public RFC process. Stage 2 (post-1.0): elected technical steering committee; foundation holds trademark/domain. |
| **Naming** | 0 → 1 | Component naming decision. |

---

## Risks and their mitigations

| Risk | Severity | Mitigation |
|---|---|---|
| **Driver availability** is the existential risk | Critical | VM-first (decided); narrow hardware list; driver VMs for hard hardware; native drivers one class at a time |
| **Ecosystem / "nobody comes"** (Fuchsia is the cautionary tale) | Critical | Ship usable value each phase; SDK from day one; greenfield starting areas; honest about cannibalization |
| **Scope explosion** (kernel + desktop + distributed + AI at once) | High | Model-first sequencing; explicit non-goals; phase gates |
| **Powerbox usability** — capability systems have foundered on the human interface | High | User testing by Phase 3; a negative result changes the design, not the narrative |
| **Overclaiming assurance** | High | "Verification-oriented" until proven; published audits including unfixed findings |
| **seL4 licensing interaction** (seL4 is GPLv2-only) | Medium | Open core (D-012) already isolates seL4 as a separate, non-resold component behind published interfaces; confirm no GPLv2 code is copied into MPL-2.0 components (driver-VM-only rule) |
| **GPU acceleration vs. small TCB** | Medium | May be no way to have both; say so honestly if true |

---

## Effort and cost

Deliberately left out. Bivdi is sequenced by dependency and gated by outcomes; staffing and cost are planning questions to revisit once the first phases prove the model, not numbers to commit to before any code exists.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
