# Bivdi — Roadmap

**Status:** Concept / planning. This roadmap is a proposal synthesized from the workgroup documents and the decisions recorded in `README.md`. Only the items marked *Decided* in `README.md` §16 are binding. No timeline is estimated here — phases are ordered by dependency, not by date.

**How to read this document:** each phase lists its *goal*, *scope*, *deliverables*, *exit gate*, and *dependencies*. An exit gate is the objective, measurable condition that must be met before the next phase begins — not an aspiration. Phases are ordered by dependency and gated by outcomes, not by schedule.

**Process:** resolving an open question in `README.md` §16–17, or making any non-trivial design decision, is recorded as an RFC in `rfcs/` *before* work that depends on it begins.

---

## Guiding principles for sequencing

1. **Build the model first, the kernel second.** The differentiator is the object/capability/state model, not the kernel. The kernel is an implementation detail.
2. **Every phase must ship value on its own.** A plan whose only payoff is at the final phase dies before it gets there.
3. **VM-first for hardware.** The hardest driver problems (GPU, Wi-Fi, suspend) are deliberately deferred behind the first cloud/VM milestone.
4. **Two tracks, one contract.** Bivdi Runtime (Linux) and Bivdi Core (microkernel) advance in parallel against a shared IDL/API.
5. **Security is a gate, not a feature.** Each phase has a security-relevant exit gate (fuzzing, audit, provenance).

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

**Dependencies.** None (started from `README.md` + `ARCHITECTURE.md`).

**Open questions to resolve before/during this phase:** IDL choice and wire format; the exact object-store mutation primitive (`cas`) semantics; the capability runtime's attenuation and revocation model at the runtime level.

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

**Security gate.** Kernel syscall surface under continuous fuzzing; capability-derivation model model-checked for authority leakage.

**Dependencies.** Phase 0 IDL/API stable enough to port to Core.

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

**Exit gate.** At least one external organization runs production workloads on Bivdi in a public cloud, with the same security guarantees as in a VM.

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
| **Attribution & naming** | 0 → 1 | Sámi language review of the name; trademark registration; component naming decision. |

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
