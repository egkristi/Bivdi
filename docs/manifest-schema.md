# Bivdi — Workload manifest (conceptual model)

**Status:** Draft. This document describes the **decided** conceptual model of the workload manifest — the human-readable security boundary. The exact schema, field names, and encoding are **open**; component naming is **deferred** (`P-003`).

---

## 1. Purpose

The manifest is the security boundary in human-readable form. A competent reviewer should be able to read one manifest and state exactly what the workload can do. If they cannot, the design is wrong regardless of its formal properties.

Decided principle: **least authority by construction** — the default grant is nothing; every grant is a diff a reviewer can see.

---

## 2. Decided content

A manifest declares, at minimum:

### 2.1 Identity and code

- **Identity** of the component (not a path — a content hash, so the running code is named by what it *is*, not where it came from).
- **Code** — the immutable, content-addressed artifact to load.

### 2.2 Capabilities

- The explicit list of capabilities the component receives at instantiation. Each is typed (what kind of resource) and scoped (which object, subtree, endpoint, device).
- **Nothing implicit.** No wildcards, no inherited handles, no environment-derived authority. A component receives nothing it does not declare, and nothing arrives later that it did not accept over an endpoint it was given.

### 2.3 Declared absences

- Explicitly declared *denials* (e.g., "no network", "no storage writes", "no spawning", "coarse clock only"). Absences are enforced, not merely documented.

### 2.4 Resource budgets

Decided budgets, all explicit and enforced:

- **memory** (hard ceiling),
- **CPU time** (a scheduling budget — time is itself a capability),
- **energy**,
- **latency class** (realtime / interactive / batch / background),
- **I/O bandwidth**.

There are no unbounded queues; an exhausted budget degrades predictably.

### 2.5 Supervision policy

- A declared restart policy. Failure is expected and local; a crashed component is restarted by its supervisor, not a system reboot. Driver restart is bounded to a performance target (`ARCHITECTURE.md` §20).

### 2.6 Provenance

- The source, builder, and reproducibility information that lets the provenance log attribute every write to a specific code hash. *(Reproducible-build thresholds are **proposed**, pending the packaging decision.)*

---

## 3. Security properties the manifest must guarantee

1. **Default-deny.** A manifest that grants nothing gives nothing.
2. **Readable authority diff.** A change that widens authority is visible as a single diff a human can review.
3. **No ambient authority.** The manifest is the *complete* enumeration of what the component can name.
4. **Budgets are capabilities, not limits to be negotiated.** A hard ceiling is a fault when exceeded, not a hint.

---

## 4. Open questions

1. **Exact schema and encoding** — field names, types, and wire/serialization format.
2. **Component naming** (`P-003`) — the manifest uses generic terms until naming is decided.
3. **Kernel-specific fields** — anything tied to the kernel capability model is blocked on the kernel decision (`P-001`).
4. **Reproducible-build/rebuilder thresholds** — proposed, tied to the packaging/update decision.

Resolution of these must be recorded as an RFC in `rfcs/` and reflected in `README.md` §16–17.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
