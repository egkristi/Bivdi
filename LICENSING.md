# Bivdi — Licensing

**Status:** Decided strategy (`D-012`). This document states the licensing model, optimized for **maximal reach and commercial upside (sale)**. The exact license texts and the interface exception should be finalized with an IP lawyer; the structure below is authoritative.

> **Not legal advice.** This document describes intent and structure. The CLA (not used), the interface exception, and any commercial terms must be drafted with a qualified intellectual-property lawyer before the first external contribution or the first sale.

---

## 1. The model: open core

Bivdi uses an **open-core** model — the strongest structure for reconciling two goals that usually conflict:

| Goal | Mechanism |
|---|---|
| **Maximum reach** | Freely implementable specification; permissive SDKs; OSI-approved weak copyleft on the reference implementation; DCO (no CLA). |
| **Commercial upside** | A proprietary commercial/enterprise layer, sold under commercial licenses; certification and the trademark as a moat. |

Open core avoids the main trade-off of a pure AGPL + CLA model: AGPL is banned in many regulated/enterprise environments (which are exactly Bivdi's customers), and a CLA raises the contribution barrier and signals single-vendor control. With open core, the *open* part is never relicensed for money — so no copyright assignment is needed — and revenue comes from proprietary layers that live alongside it.

---

## 2. What is licensed how

| Component | License | Rationale |
|---|---|---|
| **Specification** | Freely implementable (no copyright restriction on the interfaces) | Anyone may write a conforming implementation; keeps the ecosystem open and the standard valuable. |
| **SDKs, libraries, ABI headers** | **MIT OR Apache-2.0** (dual) | Maximum adoption. Dual-permissive is compatible with GPLv2-only and GPLv3, so nothing is shut out. |
| **Bivdi's own core** (services, drivers, kernel code Bivdi writes) | **MPL-2.0** | Weak, file-level copyleft: modifications to Bivdi's own files must be shared back, but applications are not infected. Low enterprise friction, OSI-approved, GPL-compatible. |
| **Third-party components** (e.g., seL4 if reused) | **Their own licenses** (seL4 is GPLv2-only) | Bivdi cannot relicense or sell what it does not own. These remain separate components. |
| **Documentation** | **CC BY 4.0** | Docs are for reading and reuse with attribution. |
| **Commercial / enterprise layer** | **Proprietary** | This is the commercial upside: management console, attestation service, certified builds, enterprise support, and closed value-add features. |

### 2.1 "Bivdi Core" means Bivdi's own code

The phrase *Bivdi Core* refers exclusively to **code Bivdi itself owns**. seL4 (GPLv2-only) and any other third-party code retain their own licenses and are used as separate components that communicate with Bivdi through published interfaces. Bivdi does not and cannot offer a commercial license to seL4 or any code it does not own.

---

## 3. No CLA — DCO

Bivdi uses a **Developer Certificate of Origin (DCO)** with a sign-off, **not** a Contributor License Agreement.

- A CLA is only necessary when the licensor needs to relicense the contributed open code under a *different* open license — which open core does **not** require, because the open core is not sold for relicensing.
- DCO keeps the contribution barrier low, which serves maximum reach.
- The proprietary commercial layer is written **only by Bivdi** (or by people who have signed an employment/contract assignment to Bivdi), so no community-contributed code ever enters the proprietary layer.

---

## 4. Interface exception (the "syscall note")

In the spirit of the Linux kernel's syscall note, Bivdi states:

> Software that merely *uses* Bivdi through its published interfaces — IPC, the IDL, WASI, and system calls — is **not** a derivative work of the Bivdi reference implementation, and its license is unaffected by MPL-2.0.

This prevents the concern that "every workload running on Bivdi is affected by copyleft", which would otherwise kill adoption faster than any technical defect. The exact wording must be confirmed with an IP lawyer.

---

## 5. Linux driver reuse — a hard rule

Linux kernel driver code is **GPLv2-only**. Therefore:

- Linux driver code may **not** be ported into Bivdi's own (MPL-2.0) components.
- Reuse of Linux drivers happens **exclusively inside driver VMs** — a deprivileged Linux guest that is a separate, isolated work, not a derivative of Bivdi's code.
- This is recorded as a rule in `AGENTS.md` so that agents and contributors never accidentally copy GPL code into the Bivdi tree.

---

## 6. AI-generated code policy

Machine-generated code has uncertain copyright protection in several jurisdictions (e.g., the United States requires human authorship). Because Bivdi sells commercial licenses to *its own* code, it must be able to assert ownership over everything in the proprietary layer. Therefore:

- All committed code is **reviewed and edited by a human** before it lands.
- The provenance of authorship is documented (who wrote/reviewed what).
- Purely machine-generated code is **not** placed in the proprietary commercial layer unless a human has substantively authored or transformed it.

---

## 7. Trademark and certification as the moat

A freely implementable specification invites competing implementations. The durable moat is therefore **trademark + certification**, not copyright alone:

- The name **Bivdi**, the wordmark, and `bivdi.com` are held by the project and transferred to a foundation at Stage 2.
- **"Bivdi-verified"** is reserved and may only be applied to configurations whose verification/assurance claims the project has confirmed.
- Conforming implementations may say **"bivdi-compatible"** after passing the conformance suite.

This means the commercial upside has three legs: (1) proprietary enterprise features, (2) certification/attestation services, and (3) support and SLAs — none of which require copyleft or a CLA.

---

## 8. Summary statement

> Bivdi is open source. The Bivdi specification is freely implementable, and the SDKs are permissively licensed (MIT OR Apache-2.0). The reference implementation of Bivdi's own code is MPL-2.0; software that merely uses Bivdi's published interfaces is not affected. The commercial enterprise layer, certification, and support are offered under separate commercial terms.

---

## 9. Open items (to finalize with counsel)

- [ ] Exact interface-exception wording.
- [ ] DCO sign-off mechanics (e.g., `git` `Signed-off-by`).
- [ ] Employment/IP-assignment terms for anyone writing the proprietary layer.
- [ ] Trademark filing (Norwegian Patentstyret, EUIPO, USPTO, classes 9 and 42).
- [ ] Choice of MPL-2.0 with or without the "secondary license" notice (MPL-2.0 §3.3).
