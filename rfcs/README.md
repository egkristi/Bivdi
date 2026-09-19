# RFCs

Every non-trivial design decision is recorded here **before** work that depends on it begins (`CONTRIBUTING.md` §5). An RFC states the decision, the alternatives rejected, its effect on the threat model, its effect on the trusted computing base, and how it is introduced without breaking the decided model.

Copy [`0000-template.md`](0000-template.md) to `NNNN-slug.md` to start one.

| RFC | Subject | Status | Resolves |
|---|---|---|---|
| [0001](0001-target-niche.md) | First target niche: the agent execution host | Accepted | `P-002` |
| [0002](0002-interface-definition-language.md) | Interface definition language and wire format | Accepted | IDL choice; unblocks `docs/abi.md` |
| [0003](0003-platform-guarantees.md) | Platform guarantees matrix: reconciling `D-002` with `D-006` | Deferred | An unrecorded conflict between two decided items |
| [0004](0004-kernel-choice.md) | Kernel choice: reuse seL4 | Deferred | `P-001` |

## Status values

- **Proposed** — written, under discussion, binding on nothing.
- **Accepted** — decided. `docs/decisions.md` and `README.md` §16–17 are updated in the same change.
- **Deferred** — not decided; parked. It may be re-opened later and is kept as the recorded analysis for when it is.
- **Rejected** — kept, not deleted. The history of a decision is part of the decision.
- **Superseded** — replaced by a later RFC, which is named here.

## Ordering

`P-002` (the target niche) was resolved first and `P-001` (the kernel) deferred, consistent with the original ordering rule: the niche determines what the kernel must support. With the Runtime as the product, the kernel is not on the critical path at all.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
