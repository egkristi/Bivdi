# Contributing to Bivdi

**Status:** Applies from day one. These rules reflect the project's decided working practices; they are not architectural decisions.

Bivdi's product is the **Bivdi Runtime** under `runtime/` — a running implementation of the six-primitive model (object, capability, state, event, identity, agent) plus the WASI runtime. The microkernel **Bivdi Core** is a parked research track (`P-001` deferred). Contributions today are primarily documentation, specifications, RFCs, and the runtime. The process below applies throughout.

---

## 1. First, read the canonical documents

- [`README.md`](README.md) — the authoritative overview, decisions (§16), and open questions (§17).
- [`AGENTS.md`](AGENTS.md) — guidance for agents and contributors, including the core invariants that must never be broken.
- [`ARCHITECTURE.md`](ARCHITECTURE.md) — technical elaboration.
- [`docs/README.md`](docs/README.md) — what is documented and what is still pending.

## 2. Everything starts from an issue

Every change is tracked by a GitHub issue. Create the issue first, then branch from it. **No change without an issue.**

If you are reporting a problem or proposing a change, the issue is the place to describe *why* before any work begins.

## 3. Branch and submit via MR

- `main` is **protected**. Never push directly to `main`.
- Create a short-lived, single-purpose branch named with its issue ID: `<type>/<issue-id>-<short-slug>` (e.g., `docs/5-agents-ci-gated-merges`, `fix/12-object-store-cas`).
- One branch, one issue, one merge/pull request (MR/PR).
- Open the MR into `main`; reference the issue with `Closes #N`.
- An MR without a linked issue is incomplete.

## 4. Commit discipline

- Commit after major changes, completed features, or new versions/releases.
- Use **Conventional Commits**: `type(scope): summary`, where `type` is `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, or `ci`. Reference the issue (e.g., `docs(workflow): enforce CI-gated merges (#5)`).
- **Sign off (DCO).** Add `Signed-off-by:` to every commit (`git commit -s`), per `LICENSING.md` §3. Verified in CI.
- Push the branch after committing and open the MR.
- Do not batch unrelated work into one branch or MR.
- Do not commit throwaway or generated artifacts; keep `.gitignore` accurate (`temp/` is currently excluded).

## 4a. CI gate

- Merging is **blocked until all required status checks pass** — this is enforced by branch protection on `main`.
- Never merge a red or pending build, and never bypass the gate. If a check is flaky or misconfigured, fix the pipeline or file an issue.

## 5. Design decisions are RFCs

Record every non-trivial design decision as an RFC in `rfcs/`, with motivation, security analysis, TCB impact, and migration plan. Kernel-growing or authority-widening changes carry the burden of proof. See `rfcs/0000-template.md`.

A decision that resolves a `P-*` or open question in `README.md` §16–17 must also update the README and `docs/decisions.md`.

## 6. Respect the decided/undecided boundary

- **Certain (decided)** items are binding — work that contradicts them is wrong.
- **Proposed/open** items are *not* decided. Do not present them as facts; mark them as proposed.
- **Component naming is deferred.** Use generic, descriptive terms ("object store", "capability runtime", "state engine", "event bus", "identity service", "agent host"), not codenames, until a naming decision is recorded.

## 7. Security and attribution

- Every new capability, syscall, or manifest field must be traceable to the threat model (`docs/threat-model.md`). Changes that widen authority are security changes and require explicit rationale.
- The Sámi name carries its meaning (`README.md` §2). Do not present it as invented or as generic "Nordic" branding.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
