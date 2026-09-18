# Bivdi — Resources and Scheduling

**Status:** Draft, grounded in the decided model. Workloads, budgets, and time-as-a-capability are decided. Scheduling-criticality bands and donation semantics are *proposed* detail.

---

## 1. Decided principle

The execution unit is a **workload**: immutable code + an explicit capability set + a resource budget. Isolation that containers provide on Linux is built in.

---

## 2. Budgets everywhere

Every workload and request has explicit, enforced budgets:

- **memory** (hard ceiling),
- **CPU time** (a budget, not an advisory limit),
- **energy**,
- **latency class** (realtime / interactive / batch / background),
- **I/O bandwidth**.

There are **no unbounded queues**. An exhausted budget degrades predictably rather than hanging the machine. A hard ceiling is a *fault* when exceeded, not a hint to be negotiated.

---

## 3. CPU time is a capability

Time is a first-class resource granted as a capability. A scheduling context `(budget, period, criticality)` is held, not merely requested; admission control at bind time rejects oversubscription, so reservations are guaranteed rather than hoped for.

*(The specific criticality bands — hard/soft/best-effort — and the `donate` handover mechanism are **proposed** and belong to the kernel decision.)*

---

## 4. Heterogeneous silicon

CPU, GPU, NPU, and crypto accelerators are **peers** in the resource model, not I/O peripherals. In practice, GPUs and NPUs carry their own (often closed) firmware and driver stacks; initially accelerators are exposed as capabilities from their driver domains, with a per-device budget.

---

## 5. Workloads, not processes

The workload replaces the process as the conceptual unit. It has identity, desired state, resource requirements, capabilities, dependencies, placement, and health.

Execution environments (decided): WASI, native, Linux ABI (user-space translation), micro-VM.

---

## 6. Open questions

- The exact scheduling bands and donation semantics (blocked on the kernel decision, `P-001`).
- Heterogeneous-silicon scheduling policy beyond "exposed as capabilities".

---

*Bivdi — nothing has ambient authority. Everything must ask.*
