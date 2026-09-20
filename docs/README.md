# Bivdi — Documentation

This directory holds the canonical technical documentation. Everything here is grounded in the decisions recorded in [`decisions.md`](decisions.md); the items marked *Proposed/open* are noted as such. The normative specification lives at the top level in [`SPEC.md`](../SPEC.md); these files elaborate its decided model.

## What is certain vs. not

- **Certain (decided):** the name and attribution, the security model (no ambient authority, capability-based, IOMMU required, crypto-shredding), the data/state/event model, WASI + Rust + language-neutral IDL, the two-track strategy, the VM-first driver target, no machine-state persistence, performance-as-a-first-class-attribute, open-core licensing, and container-friendly-not-primitive.
- **Proposed (not yet final):** kernel choice (seL4 proposed), target niche, component naming, governance/RFC process.

**Component naming is deferred.** Use generic, descriptive terms ("object store", "capability runtime", "state engine", …). No codenames until a naming decision is recorded.

## Documents

| Document | Status | Content |
|---|---|---|
| [`threat-model.md`](threat-model.md) | ✅ Present | Adversaries, trusted computing base, mitigations, out-of-scope. |
| [`attribution.md`](attribution.md) | ✅ Present | The name, its meaning, and pronunciation. |
| [`decisions.md`](decisions.md) | ✅ Present | Decision log (ADRs) for the decided items and the open proposals. |
| [`glossary.md`](glossary.md) | ✅ Present | Terminology used across the project. |
| [`abi.md`](abi.md) | 📄 Draft | ABI requirements — the *decided* principles. Open: which IDL and its encoding. |
| [`manifest-schema.md`](manifest-schema.md) | 📄 Draft | Workload manifest — the *decided* conceptual model. Open: schema/encoding, naming. |
| [`object-store-format.md`](object-store-format.md) | 📄 Draft | Object-store model — the *decided* primitives. Open: on-disk encoding, hash algorithm. |
| [`token-format.md`](token-format.md) | 📄 Draft | Durable authority token — **proposed** model. Open: encoding, naming, kernel boundary. |
| [`compatibility.md`](compatibility.md) | ✅ Present | Four-level compatibility model and the cannibalization response. |
| [`networking.md`](networking.md) | ✅ Present | Identity-based networking, flow capabilities, distribution. |
| [`ai-agents.md`](ai-agents.md) | ✅ Present | Agents as constrained citizens; the delegation model. |
| [`identity.md`](identity.md) | ✅ Present | Identity kinds, petnames, selective disclosure. |
| [`events.md`](events.md) | ✅ Present | Typed first-class events and correlation identity. |
| [`provenance.md`](provenance.md) | ✅ Present | Authority-only audit log, integrity, query model. |
| [`resources.md`](resources.md) | ✅ Present | Workloads, budgets, time-as-a-capability. |
| [`boot-attestation.md`](boot-attestation.md) | ✅ Present | Measured boot and remote attestation. |
| [`observability.md`](observability.md) | ✅ Present | Built-in metrics, tracing, correlation identity. |
| [`performance.md`](performance.md) | ✅ Present | First-class performance; reference targets (proposed). |
| [`recovery.md`](recovery.md) | ✅ Present | Declarative recovery and migration; no machine-state persistence. |
| [`capabilities.md`](capabilities.md) | ✅ Present | The capability model: properties, two tiers, powerbox, leases. |
| [`state-engine.md`](state-engine.md) | ✅ Present | Declarative desired state, generations, reconciliation. |
| [`drivers.md`](drivers.md) | ✅ Present | Driver strategy, VM-first target, tiers, isolation. |

## Rule

A document that asserts a not-yet-decided choice as fact is wrong. When a pending document is written, it must open by stating which open questions (from [`SPEC.md`](../SPEC.md) §15) it resolves, and the resolution must be recorded as an RFC in [`rfcs/`](../rfcs/README.md) and reflected in [`decisions.md`](decisions.md) and [`SPEC.md`](../SPEC.md).
