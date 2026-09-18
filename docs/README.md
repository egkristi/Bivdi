# Bivdi — Documentation

This directory holds the canonical technical documentation. Everything here is grounded in the decisions recorded in [`README.md`](../README.md) §16; the items marked *Proposed/open* are noted as such.

## What is certain vs. not

- **Certain (decided):** the name and attribution, the security model (no ambient authority, capability-based, IOMMU required, crypto-shredding), the data/state/event model, WASI + Rust + language-neutral IDL, the two-track strategy, the VM-first driver target, no machine-state persistence, and performance-as-a-first-class-attribute.
- **Proposed (not yet final):** kernel choice (seL4 proposed), target niche, component naming, license model, governance/RFC process.

**Component naming is deferred.** Use generic, descriptive terms ("object store", "capability runtime", "state engine", …). No codenames until a naming decision is recorded.

## Documents

| Document | Status | Content |
|---|---|---|
| [`threat-model.md`](threat-model.md) | ✅ Present | Adversaries, trusted computing base, mitigations, out-of-scope. |
| [`attribution.md`](attribution.md) | ✅ Present | Sámi name meaning, pronunciation, and commitments. |
| [`decisions.md`](decisions.md) | ✅ Present | Decision log (ADRs) for the decided items and the open proposals. |
| [`glossary.md`](glossary.md) | ✅ Present | Terminology used across the project. |
| [`spec.md`](spec.md) | 📄 Draft | Specification v0.1 — the decided model. Open: IDL, object-store mutation semantics, capability-runtime revocation. |
| [`abi.md`](abi.md) | 📄 Draft | ABI requirements — the *decided* principles. Open: which IDL and its encoding. |
| [`manifest-schema.md`](manifest-schema.md) | 📄 Draft | Workload manifest — the *decided* conceptual model. Open: schema/encoding, naming. |
| [`object-store-format.md`](object-store-format.md) | 📄 Draft | Object-store model — the *decided* primitives. Open: on-disk encoding, hash algorithm. |
| [`token-format.md`](token-format.md) | 📄 Draft | Durable authority token — **proposed** model. Open: encoding, naming, kernel boundary. |

## Rule

A document that asserts a not-yet-decided choice as fact is wrong. When a pending document is written, it must open by stating which open questions (from `README.md` §17) it resolves, and the resolution must be recorded as an RFC in `rfcs/` and reflected in `README.md` §16–17.
