# Bivdi — Recovery and Migration

**Status:** Draft, grounded in the decided model. "Recoverable by design" and "no machine-state persistence" (`D-007`) are decided.

---

## 1. Decided principle

**Recoverable by design.** The system supports coherent snapshots and declarative recovery. A system can be restored to a prior state without manual rebuilding.

---

## 2. Declarative recovery

Recovery is expressed declaratively, e.g.:

```text
Restore system generation = 42
Restore user data = snapshot X
Restore application state = snapshot Y
```

Rollback to any prior generation is a pointer change (immutable generations).

---

## 3. Migration

Migration moves **objects, identities, policies, workloads, and desired state** — not a manual machine rebuild. The same declarative descriptions recreate an environment elsewhere.

---

## 4. No machine-state persistence

Bivdi persists **data and configuration**, not running instruction-level machine state. A reboot yields clean execution state over consistent data. Services may take periodic checkpoints, but a restart always produces a clean runtime state over consistent data — "restart fixes it" is a property, not a bug.

---

## 5. Crypto-shredding and recovery interplay

Because history is immutable, **deletion** is key destruction (`D-010`), while **recovery** is pointer-to-a-retained-root. The two are complementary: you can restore to any *retained* generation, and securely delete anything else by destroying its key.

---

## 6. Open questions

- The exact snapshot format and checkpointing policy for services (belongs to spec v0.1).

---

*Bivdi — nothing has ambient authority. Everything must ask.*
