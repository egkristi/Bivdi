# Bivdi — Decision Log

**Status:** Authoritative for the *Decided* items. *Proposed* items are open and recorded here only for tracking.

This is the single source of truth for what is and is not decided. It mirrors `README.md` §16. Any change to a decided item, or a decision resolving a proposal or an open question in `README.md` §17, must be recorded as an RFC in `rfcs/` and reflected here.

---

## Decided

| ID | Decision | Rationale |
|---|---|---|
| **D-001** | Name is **Bivdi**; domain is **bivdi.com** | Short, unique, diacritic-free; the double meaning (hunt/fish *and* ask) captures the capability model; domain secured. See `attribution.md`. |
| **D-002** | First driver target is the **most common VM engines** (KVM/QEMU, VirtualBox, VMware, Firecracker, Proxmox, then AWS/GCP/Azure) | `virtio` is the common denominator; VM-first gives a short, well-documented driver list and avoids the hardest desktop-hardware problems (GPU, Wi-Fi, suspend). |
| **D-003** | Two parallel tracks: **Bivdi Runtime** (Linux) and **Bivdi Core** (microkernel), sharing one IDL/API | Runtime proves the model and gives an SDK fast; Core is the product. One contract means code written for one runs on the other. |
| **D-004** | **WASI components** as the native application format | WASI is already capability-oriented; the component model gives typed, language-neutral interfaces; Rust, C/C++, Go, Swift, C# already target it. |
| **D-005** | **Rust** for services and drivers; **language-neutral IDL** for protocols | Memory safety where failures are most dangerous; the wire format is the contract, not a language ABI (which also solves Rust's unstable ABI). |
| **D-006** | **IOMMU required**; hardware without IOMMU is unsupported | A device can only DMA into buffers it was explicitly given; without an IOMMU the driver-isolation model does not hold. |
| **D-007** | Files survive as export format and contracts; **no persistence of running machine state** | The file is a stable contract, not the storage model; persisting instruction-level state would persist corruption and defeat "restart fixes it". |
| **D-008** | Distribution is uniform, but **network failure/latency is never hidden** | Same API and capability model locally and remotely, but latency, timeouts, and partial failure are explicit in the types (Waldo et al., 1994). |
| **D-009** | GPU and Wi-Fi run in deprivileged **driver VMs** | Their driver stacks are enormous and Linux-specific; isolation behind an IOMMU is the pragmatic answer (sledgehammer pattern). |
| **D-010** | **Crypto-shredding** for deletion in versioned storage | History is immutable, so deletion is key destruction; satisfies "right to be forgotten" without rewriting history. |
| **D-011** | **Performance is a first-class attribute** | Performance is designed in, measured, and regression-gated; never won by weakening the security model. |
| **D-012** | **Open-core licensing** | Freely implementable spec; permissive SDKs (MIT OR Apache-2.0); MPL-2.0 for Bivdi's own core/services/drivers; proprietary commercial/enterprise layer; DCO (no CLA); interface exception. See `LICENSING.md`. |

---

## Proposed (not yet final)

| ID | Proposal | Blocking question |
|---|---|---|
| **P-001** | **seL4** as the microkernel (alternative: original microkernel, seL4 methodology) | Reuse a formally verified kernel vs. license (GPLv2) and control |
| **P-002** | First target market / niche | Headless agent/server host vs. high-security workstation vs. personal node |
| **P-003** | Component naming scheme | Evocative names vs. descriptive daemon-style names vs. deferred |
| ~~P-004~~ | ~~License model~~ → **resolved** as D-012 (open core) | See `LICENSING.md` |
| **P-005** | Governance structure and RFC process | Technical-lead + RFC now; elected committee + foundation post-1.0 |

---

## Convention

- **Decided** items are binding; work that contradicts them is wrong.
- **Proposed** items are not binding; treat them as leading options, not facts.
- A decision that resolves a proposal or an open question updates `README.md` §16–17, this file, and any affected documents in `docs/`.
