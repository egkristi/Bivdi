# RFC 0004 — Kernel choice: reuse seL4

**Status:** Deferred
**Issue:** #44
**Resolves:** `P-001` (kernel choice)
**Date:** 2026-09-19

---

## 1. Summary

Resolve `P-001` as **reuse of seL4** as Bivdi Core's kernel, used as a separate, unmodified upstream component rather than as code Bivdi owns or relicenses. This is evaluated against the headless-host niche (RFC 0001) and the platform-guarantees matrix (RFC 0003), and it is compatible with the open-core licensing decision (`D-012`), which already isolates third-party code behind published interfaces.

**Deferral note (2026-09-19).** `P-001` is **deferred indefinitely**. The strategic pivot makes the Bivdi Runtime the product and parks Bivdi Core as a research track, so the kernel choice is removed from the critical path. This RFC is retained as the leading proposal (seL4) and the recorded analysis of the alternatives, so the decision can be taken quickly if and when the Core track is reactivated. It is **deferred, not rejected** — and until it is decided, RFC 0002 §3.5 (no seL4 concept enters the IDL) keeps the door open.

## 2. Motivation

`P-001` is the one open decision that gates the *kernel half* of Phase 1: "Bivdi Core booting on the microkernel as a VM guest." The Runtime track (all six primitives) is complete and does not depend on it, which is exactly why the decision has been deferrable — but Phase 1's remaining deliverable is impossible without it.

It is also the decision with the highest downside if gotten wrong in either direction: choosing to *write* a kernel spends years and produces something worse than seL4; choosing seL4 without handling its GPLv2-only license correctly would put GPL code inside an MPL-2.0 component, contradicting the rule `AGENTS.md` states as a core invariant.

## 3. Proposal

**Use seL4, unchanged, as a separate component.**

- Bivdi Core's kernel is **seL4**, upstream, unmodified, in its own component. seL4 retains its GPLv2-only license.
- Bivdi's *own* code — the root supervisor, services, drivers, the runtime's ported services — remains MPL-2.0 and communicates with seL4 only through seL4's published interfaces (its system-call surface and capability model).
- Bivdi never forks seL4, never includes GPL code inside an MPL component, and never offers a commercial license to seL4 (it cannot; `LICENSING.md` §2.1 already states this).

### 3.1 Why reuse rather than write

The numbers are decisive. seL4 is ~8,700 lines of C and ~600 lines of assembly. The kernel took ~2.2 person-years to develop; the functional-correctness proof took ~20 person-years including tooling. Reusing it eliminates the single largest, highest-risk cost in the roadmap — a verified kernel — and substitutes a component that already has machine-checked functional-correctness proofs down to machine code on supported architectures, plus integrity and confidentiality results.

Writing an original microkernel with seL4 methodology is the stated alternative. It is attractive only if the license is unacceptable or if Bivdi needs kernel changes seL4 upstream would reject. Neither is true for the niche:

| Need (from RFC 0001) | seL4 support |
|---|---|
| VM-guest operation | Yes (aarch64, x86_64, riscv64) |
| virtio drivers in userspace | Yes — drivers are userspace components by design |
| WASI runtime in userspace | Yes — any userspace service |
| No display server, no USB, no power mgmt | Irrelevant — seL4 does none of these in the kernel |
| Capability scheduling, IOMMU handling | Native (MCS scheduling; hardware capability derivation) |

The niche removes every feature that historically pushes a project to write its own kernel: graphics, USB, suspend, and power management are all deferred (`ROADMAP.md` Phase 4 or not at all).

### 3.2 The license is handled by `D-012`, not by this RFC

`D-012` (open core) already established the rule that resolves the GPLv2-only problem:

- Third-party components keep their own licenses and are used as separate components behind published interfaces.
- Bivdi's own core is MPL-2.0.
- The Linux-driver rule is driver-VM-only.

seL4 is exactly such a third-party component. Its GPLv2-only status does not "infect" Bivdi's MPL code because components that communicate only through a published interface are not derivative works — the same relationship GPLv3 tools have to the Linux kernel they run on. The one hard obligation is procedural: **GPL code must never be copied into an MPL component**, and CI's license-policy check should enforce this once it exists.

### 3.3 What Bivdi writes

Bivdi writes everything *above* the kernel: the root supervisor, the capability runtime bridge, the services, the drivers, the WASI host. The seL4 base is treated as a build dependency with a pinned upstream revision, like a compiler — not as part of Bivdi's licensed code.

### 3.4 Residual risk and its mitigation

- **Upstream coupling.** Bivdi depends on seL4's release cadence and its proof toolchain. Mitigation: pin a revision, track upstream deliberately, and never carry a private fork.
- **Proof toolchain accessibility.** Reproducing seL4's proofs is a research-grade effort. Bivdi does **not** need to reproduce them; it needs to *cite* them and confine its own verification claims to the MPL code it actually writes (`ARCHITECTURE.md` §19's "verification-oriented until proven").
- **"We didn't write the kernel" as a perception problem.** This is answered by the project's own thesis: the kernel is not the product; the object/capability/state model is. seL4 is the mechanism that enforces it.

## 4. Alternatives considered

**Write an original microkernel in Rust following seL4 methodology.** Gives full license control (permissive or MPL) and freedom to change the kernel. Rejected: it spends the project's scarcest resource (time, verification skill) on the one part already solved, produces a *weaker* result (no proof), and is unnecessary for the niche. Its only real advantage — license — is already handled by `D-012`.

**Adopt another microkernel (Redox, Zircon, Genode base).** Redox is Rust but unverified and POSIX-shaped; Zircon is unverified and coupled to Fuchsia; Genode is a framework, not a kernel, and itself uses seL4 (among others). None has seL4's proof. Rejected.

**Stay undecided and start kernel work anyway.** Rejected: kernel work without a decision is how a project acquires an accidental, undocumented kernel commitment.

## 5. Security analysis

seL4 is the strongest available answer to the small-trusted-core invariant. Its functional-correctness proof reaches machine code on supported architectures, and its capability model is the concrete instantiation of the decided "no ambient authority" and "capabilities are unforgeable, attenuable, revocable" properties. Choosing it *strengthens* the threat model's central claim rather than merely not weakening it.

No authority is widened by this RFC. It does place the seL4 base, the proof toolchain, and the upstream release process into the supply chain — which `docs/threat-model.md` should note — but that is a documentation change, not a widening.

The one security-relevant procedure to record: **any change that would require patching seL4 is a signal to stop and re-open this decision**, because a fork would forfeit the proof.

## 6. TCB impact

The TCB is unchanged in definition — the kernel was always in it. What changes is its *content*: the kernel becomes upstream seL4 (with its proof) plus Bivdi's own root supervisor, instead of an unproven original kernel.

Net effect: the TCB's verified portion grows dramatically for free, and the unverified portion (Bivdi's own supervisor and services) is unchanged.

## 7. Performance impact

seL4's IPC and interrupt paths are the reference points the `ARCHITECTURE.md` §20 targets were already written against. No target changes. The one number to re-validate is the **IPC round trip under 500 cycles**, which seL4 has been shown to meet; Bivdi should re-measure on its own configuration rather than inherit the claim.

## 8. Migration plan

1. Pin an upstream seL4 revision and vendor it as a build dependency (never a fork).
2. Bring up seL4 as a VM guest on `aarch64-virt` and `x86_64-q35` (the Phase 1 reference platforms, per RFC 0003's matrix).
3. Write the root supervisor (Bivdi MPL-2.0) that instantiates the composition.
4. Port the Runtime's services to Core behind the shared interface (the conformance suite from RFC 0002 is the check).
5. Add a CI license-policy gate enforcing "no GPL code inside MPL components."

## 9. Document updates

On acceptance:

- `README.md` §16 — move `P-001` from Proposed to Decided as `D-016`; §17 unchanged.
- `docs/decisions.md` — add `D-016`, strike `P-001` with a pointer to this RFC.
- `ARCHITECTURE.md` §5 ("The kernel — open decision") — resolve the open status to seL4.
- `docs/threat-model.md` — note the seL4 base and proof toolchain in the supply chain.
- `docs/drivers.md` — note seL4 as the kernel; driver isolation per RFC 0003's matrix.
- `LICENSING.md` §2.1 — cross-reference seL4 as the concrete third-party component.
- `ROADMAP.md` — Phase 1 "Bivdi Core on microkernel (seL4 proposed)" becomes "seL4 (decided)".

---

*Bivdi — nothing has ambient authority. Everything must ask.*
