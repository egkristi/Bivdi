# RFC 0003 — Platform guarantees matrix: reconciling D-002 with D-006

**Status:** Deferred
**Issue:** #39
**Resolves:** The unrecorded conflict between `D-002` (VM engines first) and `D-006` (IOMMU required)
**Date:** 2026-09-19

---

## 1. Summary

`D-002` and `D-006` cannot both hold on the stated first-target platforms. This RFC keeps both decisions intact and adds the missing piece: a **per-platform guarantees matrix** stating which decided security properties actually hold on each supported target, and a rule that no platform may be presented as supported until its row is written.

**Deferral note (2026-09-19).** With the strategic pivot to "Bivdi Runtime is the product" and the microkernel (`P-001`) deferred indefinitely, the driver-isolation and IOMMU questions this RFC scopes are no longer on the critical path. The Runtime runs on Linux, where the host kernel — not Bivdi — owns DMA isolation. This RFC is therefore **deferred**, not rejected: it becomes relevant again only if and when Bivdi Core (the microkernel track) is reactivated. Its substance — never overclaim a guarantee a platform does not actually enforce — remains a standing principle and is applied to the Runtime's own claims in the meantime.

> **Deferral consequence (2026-09-20 audit, P8).** Deferring this RFC leaves `D-002` (Firecracker and the public clouds as first targets) and `D-006` (IOMMU required) both standing as *binding* decided items that contradict each other, with the reconciliation in a deferred document. If platform guarantees are simply not relevant while the Runtime is the product — a reasonable position — then that position should be recorded here, and `D-002`/`D-006` marked as **parked alongside Core** rather than binding. As written, the three-part gap (hardware IOMMU absent on first targets, hypervisor joining the TCB, and seL4's unverified IOMMU path — P4) stays distributed across three documents, none of which states it in full.

## 2. Motivation

**`D-006`** says IOMMU is required and that hardware without one is *never supported*. The hardware-tier table in `README.md` §13 ends with that row. It is load-bearing: `ARCHITECTURE.md` §12.1 explains that a driver "receives no ability to touch memory outside its domain — which is why a compromised GPU driver is a graphics outage, not a root compromise."

**`D-002`** makes the first targets KVM/QEMU, VirtualBox, VMware, Firecracker and Proxmox, then AWS, GCP and Azure.

Several of those do not expose a **guest-visible** IOMMU. Firecracker's minimal device model has none. Cloud guests generally do not get one. Inside such a guest, a compromised in-guest virtio driver can point a virtual device at any guest-physical address: the hypervisor protects the *host*, and nothing protects the rest of the *guest*. On the very first target platform, the driver-isolation property that `ARCHITECTURE.md` presents as the reason a compromised driver is survivable **does not hold**.

The same gap applies to sealing. `ARCHITECTURE.md` §17 and `docs/boot-attestation.md` seal storage to a TPM measurement. Firecracker has no vTPM.

No document reconciles this. That is the actual defect — not the gap itself, which is a normal consequence of running on someone else's hypervisor, but the absence of any statement about it. A project whose stated rule is "a document that asserts a not-yet-decided choice as fact is wrong" should not have a decided item that quietly fails on its own first target.

## 3. Proposal

### 3.1 Keep both decisions; add a matrix

Neither `D-002` nor `D-006` changes. What changes is that **`D-006` is scoped explicitly to the boundary it governs**, and every platform carries a published row saying which guarantees are available on it.

`D-006` is restated as:

> **IOMMU required.** Bivdi requires an enforced DMA boundary between a driver and memory it was not given. On bare metal this is the platform IOMMU, and hardware without one is not supported. In a virtual machine, the guarantee is provided by a guest-visible IOMMU (`virtio-iommu` or an emulated VT-d/AMD-Vi) where the hypervisor offers one. **A platform that offers neither is supported for development only, never for a production guarantee**, and its row in the platform matrix says so.

The substance of `D-006` — "a device may DMA only into buffers it was explicitly given" — is unchanged. What is added is honesty about where that is enforced by hardware, where by a hypervisor, and where not at all.

### 3.2 The matrix

Every supported platform gets a row. The columns are the decided properties that depend on the platform rather than on Bivdi.

| Platform | Guest IOMMU | vTPM / measured boot | In-guest driver isolation | Sealed storage | Status |
|---|---|---|---|---|---|
| KVM/QEMU with `virtio-iommu` | Yes | Yes (swtpm) | **Full** | **Full** | Production target |
| KVM/QEMU without `virtio-iommu` | No | Optional | Degraded | Depends on vTPM | Development only |
| Firecracker | No | No | **Degraded** | **No** | Development and CI only |
| VMware (vIOMMU enabled) | Yes | Yes | Full | Full | Production target |
| VirtualBox | Partial | Partial | Degraded | Depends | Development only |
| Proxmox (KVM, IOMMU passthrough) | Yes | Yes | Full | Full | Production target |
| AWS / GCP / Azure guests | Generally no | Platform-specific | Degraded | Platform-specific | Phase 2 — row to be completed from measurement, not from documentation |
| Bare metal with IOMMU | Yes | Yes | Full | Full | Phase 3 target |
| Bare metal without IOMMU | — | — | — | — | **Never supported** (`D-006`) |

"Degraded" has a precise meaning and must be stated wherever it appears: *a compromised driver component is confined by the capability model and by its address space, but not by a hardware DMA boundary; it can therefore reach guest-physical memory outside its assigned buffers.*

### 3.3 The rule

1. **No platform is listed as supported until its row exists.** A row written from a vendor datasheet is provisional; a row is only final once measured on that platform.
2. **The matrix is published**, in `docs/drivers.md` and on the website, not kept in an internal note. The project's credibility rests on saying this before someone else does.
3. **A degraded platform may still be a first-class development target.** Firecracker is excellent for CI precisely because it is minimal, and Phase 1's agent-isolation gate does not depend on DMA isolation — the agent is a workload confined by capabilities, not a device. This RFC keeps Firecracker in Phase 1 for that reason.
4. **Production claims require a full row.** The Phase 2 exit gate — an external organisation running production workloads — must be met on a platform whose row says Full, or the gate's wording must name the degradation explicitly.

### 3.4 Consequence for Phase 1

Phase 1's exit gate concerns agent isolation and provenance, neither of which depends on the IOMMU. It can therefore be met on Firecracker. But the **security gate** in the same phase — "kernel syscall surface under continuous fuzzing; capability-derivation model model-checked for authority leakage" — should gain a third condition:

> The platform matrix is published, and the Phase 1 demonstration states which row it ran on.

## 4. Alternatives considered

**Drop Firecracker and the clouds from the early targets; require `virtio-iommu`.** Cleanest, and it preserves `D-006` without qualification. Rejected because it costs the fastest boot and the best CI target for a property Phase 1 does not exercise, and because the cloud platforms are the Phase 2 target regardless — the matrix would have to be written eventually anyway.

**Weaken `D-006` to "IOMMU where available".** Rejected outright. `D-006` is what makes the driver isolation story true on bare metal, and softening it to a preference converts a structural guarantee into a best effort. The problem is not that `D-006` is too strong; it is that it was never scoped to a boundary.

**Say nothing and let it be discovered.** The status quo. Rejected: the first competent reviewer finds this in an afternoon, and finding it themselves costs far more credibility than publishing it does. The project's own risk table lists "overclaiming assurance" as a High risk with the mitigation "verification-oriented until proven; published audits including unfixed findings." This is that principle applied to a platform claim.

## 5. Security analysis

This RFC **narrows** claimed guarantees to the ones that are actually enforced, which is a strengthening of the threat model rather than a weakening of the system.

`docs/threat-model.md` §4 lists **malicious peripheral (DMA)** with the answer "IOMMU with per-buffer pages and strict invalidation; no driver gets an identity-mapped window." That row is true on a full platform and false on a degraded one. It should gain a scope note.

A new row should be added for the case the matrix exposes:

| Adversary | Assumed capability | Answer |
|---|---|---|
| **Compromised driver on a platform without a guest IOMMU** | Arbitrary code in a driver component, able to program a virtual device | Confined by the capability model and its address space; **not** confined by a DMA boundary. Such platforms are development-only, and the matrix says so. |

No authority is widened. The threat model gets more accurate, and one honest limitation is recorded where an implied guarantee used to be.

## 6. TCB impact

None to the TCB itself. It does clarify a TCB *assumption*: `docs/threat-model.md` §6 states "the IOMMU enforces DMA isolation" as an explicit trust assumption. On a platform with no guest IOMMU that assumption is unmet, which means the stated TCB is incomplete for those platforms — the hypervisor joins it. That should be written down, because "the CPU and its IOMMU" reads as complete today and is not, in a VM.

## 7. Performance impact

A guest IOMMU carries a real cost: DMA mapping and unmapping with strict — not deferred — IOTLB invalidation is measurably slower than an identity mapping, and `ARCHITECTURE.md` §14.3 already requires strict invalidation because of Thunderclap.

The object-store throughput targets (≥ 85% sequential, ≥ 70% of raw 4K IOPS) should therefore be qualified by platform row. Measuring them on Firecracker, where no IOMMU cost is paid, and then publishing them as the system's numbers would be exactly the kind of accidental overclaim the matrix exists to prevent.

## 8. Migration plan

1. Add the matrix to `docs/drivers.md` with every row marked provisional.
2. Restate `D-006` per §3.1 in `docs/decisions.md` and `README.md` §16. This is a clarification of scope, not a reversal; the decision ID is retained.
3. Add the scope note and the new adversary row to `docs/threat-model.md`.
4. Add the matrix-publication condition to Phase 1's security gate in `ROADMAP.md`.
5. As each platform is brought up, replace its provisional row with a measured one, and record the measurement method.

## 9. Document updates

On acceptance:

- `docs/decisions.md` — `D-006` restated with its boundary scope; rationale updated.
- `README.md` §13 and §16 — hardware tier table gains the guarantee columns; `D-006` wording updated.
- `docs/drivers.md` — the matrix lands here as the normative version.
- `docs/threat-model.md` — §4 DMA row scoped; new adversary row; §6 trust assumptions note the hypervisor on VM platforms.
- `docs/boot-attestation.md` — note that sealing requires a vTPM and is unavailable on platforms without one.
- `docs/performance.md` — storage targets qualified by platform row.
- `ROADMAP.md` — Phase 1 security gate gains the publication condition; Phase 2 gate requires a Full row.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
