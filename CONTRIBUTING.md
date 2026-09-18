# Contributing to Bivdi

**Status:** Applies from day one. These rules reflect the project's decided working practices; they are not architectural decisions.

Bivdi is at the concept/planning stage — there is no build system and no code yet. Contributions today are primarily documentation, specifications, and RFCs. The process below still applies.

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
- Create a short-lived, single-purpose branch named after its issue (e.g., `docs/issue-based-workflow`).
- One branch, one issue, one merge/pull request (MR/PR).
- Open the MR into `main`; reference the issue with `Closes #N`.

## 4. Commit discipline

- Commit after major changes, completed features, or new versions/releases.
- Write a clear, single-purpose commit message.
- Push the branch after committing and open the MR.
- Do not batch unrelated work into one branch or MR.
- Do not commit throwaway or generated artifacts; keep `.gitignore` accurate (`temp/` is currently excluded).

## 5. Design decisions are RFCs

Record every non-trivial design decision as an RFC in `rfcs/`, with motivation, security analysis, TCB impact, and migration plan. Kernel-growing or authority-widening changes carry the burden of proof. See `rfcs/0000-template.md`.

A decision that resolves a `P-*` or open question in `README.md` §16–17 must also update the README and `docs/decisions.md`.

## 6. Respect the decided/undecided boundary

- **Certain (decided)** items are binding — work that contradicts them is wrong.
- **Proposed/open** items are *not* decided. Do not present them as facts; mark them as proposed.
- **Component naming is deferred.** Use generic, descriptive terms ("object store", "capability runtime", "state engine", "event bus", "identity service", "agent host"), not codenames, until a naming decision is recorded.

## 7. Security and attribution

- Every new capability, syscall, or manifest field must be traceable to the threat model (`docs/threat-model.md`). Changes that widen authority are security changes and require explicit rationale.
- The Sámi name carries commitments (`README.md` §2). Never present it as invented or as generic "Nordic" branding; preserve attribution text.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
