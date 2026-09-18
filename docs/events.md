# Bivdi — Events

**Status:** Draft, grounded in the decided model. "Events over polling" is a core invariant. This document elaborates the decided event model.

---

## 1. Decided principle

**Events over polling.** Changes produce structured, first-class events. Applications, automation, synchronization, and observability consume the **same** event fabric.

Events are *typed records*, not text that must be parsed. They are defined in the IDL.

---

## 2. Event classes

Representative events (not an exhaustive schema):

- **Object lifecycle:** created / changed / deleted
- **Workload lifecycle:** started / stopped / failed / restarted
- **Capability:** granted / used / revoked
- **Device:** attached / removed
- **Network:** connected / disconnected
- **Policy:** changed
- **System:** generation activated / rollback started

---

## 3. Properties

- **First-class.** Events are ordinary typed objects, addressable and queryable like anything else.
- **Typed.** A consumer can subscribe to a typed filter (e.g., "all `capability.revoked` events"), not a regex over logs.
- **Single fabric.** Synchronization, automation, and observability use the same bus — there is no separate "metrics/logging" side channel.

---

## 4. Correlation identity

A shared **correlation identity** traces a single user action through applications, IPC, services, storage, and networking. "Why is this happening?" and "what caused this change?" are queries, not log archaeology.

---

## 5. Open questions

- The concrete event schema and subscription/stream semantics — these belong to the IDL decision and spec v0.1.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
