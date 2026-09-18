# Bivdi — State Engine (Declarative State)

**Status:** Draft, grounded in the decided model. "Declarative state" and "immutable generations" are core invariants.

---

## 1. Decided principle

**State over configuration.** Users, applications, and administrators describe *desired state*; the system continuously reconciles observed state toward it.

```text
Desired State → Reconciler → Observed State → Actions → New Observed State
```

This applies to workloads, networking, storage, permissions, devices, and configuration.

---

## 2. Generations

A system update creates a new **immutable generation** instead of mutating the running system.

```text
Generation 101
Generation 102
Generation 103 ← active
Generation 104 ← candidate
```

- Activation is **transactional** and **health-checked**.
- A failed generation **rolls back automatically** to the previous one.
- Rollback is a pointer change (immutable generations).
- A new generation shares everything unchanged with the previous one.

---

## 3. Reconciliation vs. interaction

- **Reconciliation** suits configuration and workloads (eventual convergence).
- **Interactive** data uses direct, transactional operations — the user expects immediate response, not eventual convergence.

---

## 4. The machine as a value

The entire machine's state is described declaratively. `stated` computes the difference between desired and observed state and activates a new generation atomically.

---

## 5. Open questions

- The exact desired-state schema and diffing semantics (belongs to the IDL decision and spec v0.1).
- Schema evolution of state objects.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
