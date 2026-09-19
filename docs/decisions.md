# Bivdi — Decision Log

**Status:** Authoritative for the *Decided* items. *Proposed* items are open and recorded here only for tracking.

This is the single source of truth for what is and is not decided. It mirrors `README.md` §16. Any change to a decided item, or a decision resolving a proposal or an open question in `README.md` §17, must be recorded as an RFC in `rfcs/` and reflected here.

---

## Decided

| ID | Decision | Rationale |
|---|---|---|
| **D-001** | Name is **Bivdi**; domain is **bivdi.com** | Short, unique, diacritic-free; the double meaning (hunt/fish *and* ask) captures the capability model; domain secured. See `attribution.md`. |
| **D-002** | First driver target is the **most common VM engines** (KVM/QEMU, VirtualBox, VMware, Firecracker, Proxmox, then AWS/GCP/Azure) | `virtio` is the common denominator; VM-first gives a short, well-documented driver list and avoids the hardest desktop-hardware problems (GPU, Wi-Fi, suspend). |
| **D-003** | **Bivdi Runtime is the product; Bivdi Core is a parked research track.** The Runtime (Linux) is the shipped product; the microkernel Core is deferred research, not a parallel product track. One IDL/API is retained so that any future Core can run the same programs. | The Runtime reaches the target niche (agent execution host) with a small fraction of the risk of a microkernel bring-up; the Core track was consuming attention and deferring the actual product. The shared contract is kept so the Core option is not foreclosed. |
| **D-004** | **WASI components** as the native application format | WASI is already capability-oriented; the component model gives typed, language-neutral interfaces; Rust, C/C++, Go, Swift, C# already target it. |
| **D-005** | **Rust** for services and drivers; **language-neutral IDL** for protocols | Memory safety where failures are most dangerous; the wire format is the contract, not a language ABI (which also solves Rust's unstable ABI). |
| **D-006** | **IOMMU required**; hardware without IOMMU is unsupported | A device can only DMA into buffers it was explicitly given; without an IOMMU the driver-isolation model does not hold. |
| **D-007** | Files survive as export format and contracts; **no persistence of running machine state** | The file is a stable contract, not the storage model; persisting instruction-level state would persist corruption and defeat "restart fixes it". |
| **D-008** | Distribution is uniform, but **network failure/latency is never hidden** | Same API and capability model locally and remotely, but latency, timeouts, and partial failure are explicit in the types (Waldo et al., 1994). |
| **D-009** | GPU and Wi-Fi run in deprivileged **driver VMs** | Their driver stacks are enormous and Linux-specific; isolation behind an IOMMU is the pragmatic answer (sledgehammer pattern). |
| **D-010** | **Crypto-shredding** for deletion in versioned storage | History is immutable, so deletion is key destruction; satisfies "right to be forgotten" without rewriting history. |
| **D-011** | **Performance is a first-class attribute** | Performance is designed in, measured, and regression-gated; never won by weakening the security model. |
| **D-012** | **Open-core licensing** | Freely implementable spec; permissive SDKs (MIT OR Apache-2.0); MPL-2.0 for Bivdi's own core/services/drivers; proprietary commercial/enterprise layer; DCO (no CLA); interface exception. See `LICENSING.md`. |
| **D-013** | **Container-friendly, not container-primitive** | The runtime runs in a container from the first executable; containers are a deployment/compatibility concern, never an architectural or security primitive. |
| **D-014** | **First target niche is the agent execution host** | The machine you run agents on, where what an agent touched is a queryable fact and what it could touch was bounded before it started. Headless; "server host" dropped. See [`0001`](../rfcs/0001-target-niche.md). |
| **D-015** | **WIT as the IDL**; Component Model canonical ABI in-process, deterministic CBOR across boundaries. No seL4 concept enters the IDL. | WASI components are already the native format (`D-004`); WIT gives typed resources with own/borrow, generated bindings, and keeps the deferred kernel (`P-001`) an implementation detail. See [`0002`](../rfcs/0002-interface-definition-language.md). |

---

## Proposed (not yet final)

| ID | Proposal | Blocking question | RFC |
|---|---|---|---|
| **P-001** | **seL4** as the microkernel (alternative: original microkernel, seL4 methodology) — **deferred indefinitely**; Core is parked research | Reuse a formally verified kernel vs. license (GPLv2) and control | [`0004`](../rfcs/0004-kernel-choice.md) — deferred |
| ~~P-002~~ | ~~First target market / niche~~ → **resolved** as D-014 (agent execution host) | See [`0001`](../rfcs/0001-target-niche.md) | accepted |
| **P-003** | Component naming scheme | Evocative names vs. descriptive daemon-style names vs. deferred | — |
| ~~P-004~~ | ~~License model~~ → **resolved** as D-012 (open core) | See `LICENSING.md` | — |
| **P-005** | Governance structure and RFC process | Technical-lead + RFC now; elected committee + foundation post-1.0 | — |

**`P-001` is deferred indefinitely.** The Runtime is the product; the kernel is not on the critical path for the agent execution host. RFC 0002 §3.5 keeps the door open by forbidding seL4 concepts in the IDL.

---

## Open RFCs

RFCs in flight. None is binding until accepted; an accepted RFC updates this file, `README.md` §16–17, and every affected document.

| RFC | Subject | Status |
|---|---|---|
| [`0001`](../rfcs/0001-target-niche.md) | First target niche: the agent execution host | Accepted → `D-014` |
| [`0002`](../rfcs/0002-interface-definition-language.md) | Interface definition language and wire format | Accepted → `D-015` |
| [`0003`](../rfcs/0003-platform-guarantees.md) | Platform guarantees matrix | Deferred (Core track) |
| [`0004`](../rfcs/0004-kernel-choice.md) | Kernel choice: reuse seL4 | Deferred (`P-001`) |

### Decisions still owed an RFC

`CONTRIBUTING.md` §5 requires an RFC for every non-trivial design decision, *before* dependent work begins. These were taken without one and should be recorded retroactively:

- `D-001` … `D-013` — the entire decided list predates any RFC. `D-013` was added most recently, still without one. (`D-014` and `D-015` are covered by RFCs 0001 and 0002.)
- **BLAKE3** as the content-addressing hash — implemented, marked "provisional" only in a doc comment.
- ~~**`Right` as a three-value total order**~~ — **resolved** in code: `Right` is now `Rights`, a six-flag set (`read, write, execute, grant, signal, revoke`) with subset-inclusion attenuation, per RFC 0002 §5. The three-value ordered enum is gone.
- **Agent leases held separately from capability leases** — a real semantic choice, currently undocumented outside the code.
- **Opaque in-process ids** as the stand-in for unforgeability on the Runtime track.
- **The event-kind string taxonomy** in `bivdi-event`.

---

## Convention

- **Decided** items are binding; work that contradicts them is wrong.
- **Proposed** items are not binding; treat them as leading options, not facts.
- A decision that resolves a proposal or an open question updates `README.md` §16–17, this file, and any affected documents in `docs/`.
