# AGENTS.md

Guidance for AI coding agents (and human contributors) working in this repository.

## Project

**Bivdi** — a capability-secure, object-centric, declarative operating system. The canonical overview, decisions, and open questions live in [`README.md`](README.md). Read it first.

> *bivdit* (North Sámi) — to ask for, to request; also to hunt, to fish.
> In Bivdi, nothing has ambient authority. Everything must ask.

**Status:** Concept / planning. No code exists yet. Do not treat any architectural statement here as frozen except the items listed as *Decided* in the README (§16).

## Source documents

The original design ideas are the workgroup documents under `temp/` (`workgroup1.md` … `workgroup4.md`). They are **git-ignored** (`temp/` is excluded) and are unedited idea/workshop output — not authoritative specs. When in doubt, the `README.md` is the authoritative synthesis; the workgroup files are background.

## Core invariants (never break these)

These are the load-bearing design commitments. Any code, doc, or proposal that contradicts them is wrong.

1. **No ambient authority.** A component's authority is exactly the capabilities it was handed. No `root`, no `sudo`, no global namespace. A process gains nothing from *who* launched it.
2. **Objects over paths.** Data lives in a typed, content-addressed object store; the filesystem is a compatibility *view*, never the conceptual center.
3. **Declarative state.** Configuration and workloads are described as desired state and reconciled; updates are immutable generations with transactional rollback.
4. **Events over polling.** Changes produce structured, first-class events.
5. **Compatibility without compromise.** Legacy (Linux/Windows) software runs in isolated subsystems and never weakens the native model.
6. **AI without unrestricted authority.** Agents are capability-constrained, time-limited, quota-bound, and fully provenance-logged. "AI proposes, the OS enforces."
7. **Small trusted core.** Whatever must be correct to keep the system secure must be small enough to prove.
8. **IOMMU required.** Hardware without IOMMU is unsupported.

## Decided vs. open

Consult `README.md` §16. As of writing:

- **Decided:** name/domain; VM-engine-first driver target; two parallel tracks (Runtime on Linux + Core on microkernel); WASI as native app format; Rust + language-neutral IDL; IOMMU required; no persistence of running machine state; GPU/Wi-Fi in driver VMs; crypto-shredding for deletion.
- **Proposed/open:** kernel choice (seL4 is the leading proposal); target niche; component naming; license model; governance/RFC process.

**Component naming is deferred.** Use generic, descriptive terms in code and docs — "object store", "capability runtime", "state engine", "event bus", "identity service", "agent host" — not evocative codenames or daemon-style abbreviations, until a naming decision is recorded.

## Conventions (apply once code exists)

- **Languages:** Rust for services, drivers, and the kernel; C/assembly only in the trusted core where required. Application-facing code targets WASI/WASM.
- **Protocols:** define component interfaces in a language-neutral IDL; generate bindings rather than hand-writing ABI calls. The wire format is the contract, not a language calling convention.
- **Security model:** every new capability, syscall, or manifest field must be traceable to the threat model. Changes that widen authority are treated as security changes and require explicit rationale.
- **Licensing (proposed, pending decision):** kernel/ABI headers permissive (Apache-2.0 OR MIT); services/drivers MPL-2.0; docs CC BY 4.0. Do not introduce dependencies whose licenses conflict with this before the license decision is finalized.
- **Attribution:** the Sámi name carries commitments (README §2). Never present the name as invented or as generic "Nordic" branding; preserve attribution text in docs.

## Repository layout

Planned structure (from `README.md` §18); create directories as they become needed:

```text
docs/       specs, threat model, ABI, attribution
kernel/     the trusted core (microkernel)
runtime/    Bivdi Runtime on Linux
services/   object store, capability runtime, state engine, event bus,
            identity, agent host, network, storage, provenance/audit,
            packaging, compositor (later)
drivers/    one directory per driver, one isolated component each
lib/        capability-typed stdlib, syscall bindings, C ABI
compat/     Linux ABI layer, micro-VM integration
tools/      build, package, audit, image tooling
tests/      property, fault-injection, conformance, fuzz
rfcs/       all design decisions with rationale
```

## Process

- Record every non-trivial design decision as an RFC in `rfcs/`, with motivation, security analysis, TCB impact, and migration plan. Kernel-growing or authority-widening changes carry the burden of proof.
- Keep the threat model updated alongside architectural change.
- Decisions that resolve a `P-*` or open question in `README.md` §16–17 must update the README accordingly.

## Build & test

No build system exists yet. When the first code lands, establish (and document here) the canonical commands for build, test, lint, and fuzz, and keep them reproducible.
