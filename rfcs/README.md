# RFCs

Every non-trivial design decision is recorded here **before** work that depends on it begins (`CONTRIBUTING.md` §5). An RFC states the decision, the alternatives rejected, its effect on the threat model, its effect on the trusted computing base, and how it is introduced without breaking the decided model.

Copy [`0000-template.md`](0000-template.md) to `NNNN-slug.md` to start one.

| RFC | Subject | Status | Resolves |
|---|---|---|---|
| [0001](0001-target-niche.md) | First target niche: the headless agent and server host | Proposed | `P-002` |
| [0002](0002-interface-definition-language.md) | Interface definition language and wire format | Proposed | IDL choice; unblocks `docs/abi.md` |
| [0003](0003-platform-guarantees.md) | Platform guarantees matrix: reconciling `D-002` with `D-006` | Proposed | An unrecorded conflict between two decided items |

## Status values

- **Proposed** — written, under discussion, binding on nothing.
- **Accepted** — decided. `docs/decisions.md` and `README.md` §16–17 are updated in the same change.
- **Rejected** — kept, not deleted. The history of a decision is part of the decision.
- **Superseded** — replaced by a later RFC, which is named here.

## Ordering

`P-002` (the target niche) is resolved before `P-001` (the kernel). The niche determines what the kernel must support.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
