# RFC 0000 — Template

> Copy this file to `rfcs/NNNN-slug.md` for each RFC. Replace `NNNN` with the next number and `slug` with a short kebab-case title.

**Status:** Proposed (then: Accepted / Rejected / Superseded)
**Issue:** #N — the GitHub issue tracking this decision
**Resolves:** `P-XXX` or open question(s) from `README.md` §16–17 (list them)
**Date:** YYYY-MM-DD

---

## 1. Summary

One paragraph on what is being decided and why now.

## 2. Motivation

The problem this decision addresses, and why the status quo is insufficient.

## 3. Proposal

The concrete design or decision. Be precise enough that a reviewer can judge it.

## 4. Alternatives considered

What was rejected and why.

## 5. Security analysis

How the proposal affects the threat model (`docs/threat-model.md`). Does it widen authority? If so, state the explicit rationale — authority-widening changes carry the burden of proof.

## 6. TCB impact

Does this grow or shrink the trusted computing base? Justify any kernel-growing change against "it can live in userspace".

## 7. Performance impact

Effect on the performance targets in `ARCHITECTURE.md` §20, if any.

## 8. Migration plan

How the change is introduced without breaking the decided model or existing work.

## 9. Document updates

Which files must change when this is accepted: `README.md` §16–17, `docs/decisions.md`, and any affected `docs/` documents.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
