# Bivdi — Observability

**Status:** Draft, grounded in the decided model. "Observable by design" and "events over polling" are core principles. This document elaborates the decided observability model.

---

## 1. Decided principle

**Observability is built in**, not bolted on as agents you install afterward. Logs, metrics, traces, security events, capability events, and lifecycle events are native.

---

## 2. What is observed

- **Metrics** for all components and workloads.
- **Structured logs** as typed events (see `events.md`), not text.
- **Distributed tracing** with causality across IPC and the network.
- **Resource usage** per budget and capability.
- **Security events** — capability grant/use/revocation.
- **Dependency graph** between components.

---

## 3. Deterministic answers

Observability exists so that questions can be answered deterministically, for example "why is the machine slow?":

```text
CPU:    73% agent-runtime (budget 80%)
        14% object store (indexing)
Memory: 81%
I/O:    object store → nvme0, 94% (indexing after import)
Net:    agent-runtime → api.example.com, 18 MB/s
```

*(The exact rendering is illustrative; the underlying per-budget accounting is decided.)*

---

## 4. Correlation identity

A shared **correlation identity** traces a single user action through applications, IPC, services, storage, and networking. See `events.md`.

---

## 5. Deeper analysis

Programmable, verified hooks (eBPF-style) are available for deeper analysis — **always as capabilities**, never as ambient authority.

---

## 6. Open questions

- The concrete metric/trace schemas belong to the IDL decision and spec v0.1.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
