# Security Policy

**Status:** Concept/planning for the kernel, with a complete **Bivdi Runtime** under `runtime/` (all six primitives + WASI runtime) implemented and tested. There is no shipped operating system yet, so there is no production software with vulnerabilities to report. This policy states the security commitments and how to report issues against the design and the runtime.

## What is in scope

Bivdi's security goal is structural: a compromised or malicious component can only exercise the authority it was explicitly given. The canonical statement of that goal, and of what is and is not defended, is the threat model:

- [`docs/threat-model.md`](docs/threat-model.md)

The load-bearing commitments are the core invariants in [`AGENTS.md`](AGENTS.md) — including no ambient authority, objects over paths, declarative state, a small trusted core, and IOMMU required.

## Reporting

Report security concerns as GitHub issues (see [`CONTRIBUTING.md`](CONTRIBUTING.md) — every change starts from an issue). If a report concerns a live exploit or sensitive material, do **not** post it publicly; use GitHub's private vulnerability reporting on the repository instead.

## What to report

- A design that widens authority without explicit rationale.
- A proposal that contradicts a decided item (`docs/decisions.md`) or a core invariant.
- A flaw in the threat model's adversary/mitigation reasoning.
- A gap between the documentation and the decided security model.

## What is out of scope (for now)

Until code exists, the following are not yet applicable:

- Runtime vulnerabilities, CVEs, or patch deadlines — there is no software to exploit.
- Cryptographic parameters or on-disk formats — these are not yet decided.

## Security as a process

- Changes that widen authority are treated as security changes and require an RFC with explicit rationale.
- The threat model is updated alongside any architectural change.
- Assurance claims are never overstated: until a formal verification path is complete, the project is "verification-oriented", not "verified".

---

*Bivdi — nothing has ambient authority. Everything must ask.*
