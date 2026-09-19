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

- **Decided:** name/domain; VM-engine-first driver target; two parallel tracks (Runtime on Linux + Core on microkernel); WASI as native app format; Rust + language-neutral IDL; IOMMU required; no persistence of running machine state; GPU/Wi-Fi in driver VMs; crypto-shredding for deletion; performance as a first-class attribute; open-core licensing.
- **Proposed/open:** kernel choice (seL4 is the leading proposal); target niche; component naming; governance/RFC process.

**Component naming is deferred.** Use generic, descriptive terms in code and docs — "object store", "capability runtime", "state engine", "event bus", "identity service", "agent host" — not evocative codenames or daemon-style abbreviations, until a naming decision is recorded.

## Conventions (apply once code exists)

- **Languages:** Rust for services, drivers, and the kernel; C/assembly only in the trusted core where required. Application-facing code targets WASI/WASM.
- **Protocols:** define component interfaces in a language-neutral IDL; generate bindings rather than hand-writing ABI calls. The wire format is the contract, not a language calling convention.
- **Security model:** every new capability, syscall, or manifest field must be traceable to the threat model. Changes that widen authority are treated as security changes and require explicit rationale.
- **Performance is first-class:** performance-sensitive paths (IPC, object store, scheduling) have targets in `ARCHITECTURE.md` §20; regressions past them are treated as failures, never traded for weakening the security model.
- **Licensing (decided, open core):** spec freely implementable; SDKs/ABI headers MIT OR Apache-2.0; Bivdi's own core/services/drivers MPL-2.0; docs CC BY 4.0; commercial/enterprise layer proprietary. Third-party code keeps its own license. See [`LICENSING.md`](LICENSING.md).
- **Linux driver reuse is driver-VM-only.** Linux kernel driver code is GPLv2-only and must **never** be copied into Bivdi's own (MPL-2.0) components. Reuse of Linux drivers happens only inside an isolated driver VM. Do not introduce dependencies or code whose license conflicts with this.
- **AI-generated code policy.** All committed code is human-reviewed/edited before landing; provenance of authorship is documented; purely machine-generated code is not placed in the proprietary commercial layer.
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

The Bivdi Runtime (`runtime/`) is the first code in the repository.

```sh
cd runtime
cargo build --workspace     # build
cargo test --workspace      # test
cargo clippy --workspace --all-targets -- -D warnings  # lint (CI gate)
cargo fmt --all --check     # format check (CI gate)
```

- **Rust workspace** under `runtime/`, crates `bivdi-object`, `bivdi-cap`, `bivdi-state`, `bivdi-agent`, `bivdi-cli`.
- CI runs `fmt`, `build`, `clippy` (deny warnings), and `test` on every PR to `main`.
- `target/` is git-ignored; never commit build artifacts.

## Version control

Bivdi follows a GitHub issue-driven workflow. Every change traces back to an issue, every branch carries that issue's ID, every merge request links its issue, and merging is gated on CI.

### Issues

- **Everything starts from an issue.** Create the issue first; there is no change without an issue. All work is registered in GitHub Issues before any branch is created.
- **Scope is single-purpose.** One issue = one concern. Break large efforts into separate issues.
- **Use a template.** Give each issue a clear title, a description of the problem/motivation, and (where relevant) acceptance criteria and links to any open question in `README.md` §16–17.

### Branches

- **Branch from `main`, named with the issue ID.** Branch names follow `<type>/<issue-id>-<short-slug>`, e.g. `docs/5-agents-ci-gated-merges`, `fix/12-object-store-cas`, `feat/23-linux-abi-epoll`. The numeric part is the GitHub issue number — every branch carries its issue ID.
- **One branch per issue.** Short-lived and single-purpose; do not batch unrelated work into one branch.
- **Never push directly to `main`.** `main` is protected; all changes land via a merge/pull request (MR/PR).

### Commits

- **Conventional Commits.** Write messages as `type(scope): summary`, where `type` is one of `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `ci`, and `scope` is optional. Reference the issue in the message, e.g. `docs(workflow): enforce CI-gated merges (#5)`.
- **Commit after major changes**, completed features, or new versions/releases — not arbitrary intervals.
- **Don't commit** throwaway or generated artifacts; keep `.gitignore` accurate (currently `temp/` is excluded).

### Merge requests (MRs/PRs)

- **Every MR is linked to its issue.** Reference it in the description with `Closes #N` (or `Fixes #N` / `Resolves #N`) so merging the MR closes the issue. An MR without a linked issue is incomplete.
- **One MR per issue**, targeting `main`.
- **Describe the change** and how it meets the issue's acceptance criteria.
- **Merge is blocked until CI passes.** All required status checks must be green before merge. Never merge a red or pending build.

### CI/CD gate (applies once a pipeline exists)

- **Required checks must pass.** The repository's CI (lint, build, test, and any security-relevant gates) must complete successfully before a merge is possible. This is enforced by branch protection on `main`.
- **Don't bypass the gate.** If a check is flaky or misconfigured, fix the pipeline or file an issue — do not force-merge around it.
- **No build system exists yet.** When the first code lands, establish the canonical commands for build, test, lint, and fuzz, wire them into CI, and document the required checks here.
