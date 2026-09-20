# Bivdi

**The machine you run agents on — where what an agent touched is a queryable fact, and what it could touch was bounded before it started.**

> *bivdit* (North Sámi) — to ask for, to request; also to hunt, to fish.
> In Bivdi, nothing has ambient authority. Everything must ask.

- **Project:** Bivdi
- **Domain:** [bivdi.com](https://bivdi.com)
- **Status:** Milestone A in progress — the Runtime is implemented and runnable on Linux (see [`runtime/`](runtime/)), with two of the four exit-gate clauses met. The **Bivdi Runtime is the product**; the microkernel Core is a parked research track.
- **Document:** What Bivdi is.

> **Where things live now.** This file says *what Bivdi is*. The **normative specification** — what a Bivdi system MUST do — is [`SPEC.md`](SPEC.md). The **implementation architecture** is [`ARCHITECTURE.md`](ARCHITECTURE.md). The **execution plan** is [`ROADMAP.md`](ROADMAP.md). The decided/proposed boundary lives in [`docs/decisions.md`](docs/decisions.md).

---

## 1. What Bivdi is

Bivdi is a **capability-secure execution platform for AI agents**, running on Linux today. It is built around a single premise:

> **The computer should be a secure, distributed, stateful environment for people and workloads — not a pile of processes wrapped around a filesystem.**

The long-term vision is an operating system built around the same model. That is a deliberately parked research track (`D-003`) rather than a roadmap commitment — no current deliverable depends on it, and nothing here should be read as describing a kernel that exists.

*Platform* is the category; the **agent execution host** (`D-014`) is the thing you actually run: the machine you point agents at, where what an agent touched is a queryable fact and what it could touch was bounded before it started. It abandons several assumptions inherited from 1970s time-sharing systems:

- **No ambient authority.** A program has no access to anything it was not explicitly handed. There is no `root`, no `sudo`, and no user ID that grants power by virtue of who launched a program.
- **Objects over files.** Data lives in a typed, versioned, content-addressed object store. The filesystem survives as a compatibility view, not the conceptual center.
- **State over configuration.** The system is described declaratively and continuously reconciled toward desired state. Updates are immutable generations with transactional rollback.
- **Events over polling.** Changes produce structured, first-class events consumed uniformly by applications, automation, and observability.
- **Compatibility without compromise.** Existing Linux and Windows software runs in isolated subsystems that never weaken the native security model.
- **AI without unrestricted authority.** AI agents are first-class workloads, but an agent receives a narrow, time-limited, quota-bound capability set and everything it does is attributable.

Bivdi is explicitly **not** a Linux distribution, not a Kubernetes project, not a mobile OS, and not a blockchain.

---

## 2. Name

**Bivdi** is a North Sámi word derived from the verb *bivdit* ("to hunt, to fish; to ask for, to request"). The name fits the system: a hunter takes only what is needed, with the right tool — and in a capability system, everything must *ask*.

---

## 3. Core design principles

Ranked; when principles conflict, the higher one wins.

1. **No ambient authority.** All access is an explicitly granted capability.
2. **Least authority by construction.** The default grants nothing; every grant is a readable diff.
3. **Identity everywhere.** People, devices, workloads, services, objects, and organizations have identity.
4. **Objects over paths.** Typed, addressable objects replace the global namespace of the filesystem.
5. **State over configuration.** Declarative desired state, reconciled, versioned, and rollback-able.
6. **Events over polling.** Structured events are the native integration and observability mechanism.
7. **Declarative over imperative.** The system converges toward what is desired, not what a script last did.
8. **Immutable over mutable.** System generations and content-addressed data make rollback a pointer change.
9. **Small trusted core.** Whatever must be correct to keep the system secure must be small enough to prove.
10. **Compatibility without compromise.** Legacy software runs isolated; it never weakens the native model.
11. **AI foreslår, OS håndhever.** AI proposes; the OS enforces. No agent gets more than it was delegated.
12. **Portable, recoverable, observable by design.**
13. **Performance is a first-class attribute.** Performance is designed in, measured, and regression-gated — but never won by weakening the security model.
14. **Container-friendly, but containers are not a primitive.** The runtime runs in a container from the first executable, for deployment and compatibility. Containers are a compatibility view — they never become an architectural or security primitive.

---

## 4. The fundamental model

Traditional operating systems center on *processes, PIDs, UIDs, files, and sockets*. Bivdi centers on:

```text
IDENTITY + CAPABILITY + OBJECT + EVENT + DESIRED STATE + RESOURCE
```

Everything important has an identity. Authority is explicit. State is declared and reconciled. Changes produce structured events. Data and resources are addressable objects. These six primitives — not the kernel — are the defining architecture.

---

## 5. The four documents

| Document | Purpose |
|---|---|
| [`README.md`](README.md) | **What Bivdi is** — this overview. |
| [`SPEC.md`](SPEC.md) | **Normative specification** — the model, invariants, and semantics any implementation MUST satisfy, using RFC 2119 language (MUST / MUST NOT / SHOULD / SHOULD NOT / MAY). |
| [`ARCHITECTURE.md`](ARCHITECTURE.md) | **Implementation architecture** — the system diagram, the trusted core, drivers, and how the model is realized. |
| [`ROADMAP.md`](ROADMAP.md) | **Execution plan** — milestones, exit gates, and the parked Core track. |

Supplementary technical detail — the threat model, capability/state/event/identity/provenance models, compatibility, networking, performance, and more — lives under [`docs/`](docs/README.md). Design decisions and their rationale are recorded in [`docs/decisions.md`](docs/decisions.md) and the RFCs under [`rfcs/`](rfcs/README.md).

---

## 6. License

**Open core** (`D-012`). See [`LICENSING.md`](LICENSING.md) for the full model. Summary:

- **Specification:** freely implementable.
- **SDKs, libraries, ABI headers:** MIT OR Apache-2.0 (dual).
- **Bivdi's own core/services/drivers:** MPL-2.0.
- **Third-party code (e.g., seL4):** its own license (seL4 is GPLv2-only).
- **Documentation:** CC BY 4.0.
- **Commercial/enterprise layer:** proprietary, sold separately.

> seL4 (GPLv2-only) is never part of Bivdi's licensed core and its source is never conveyed as part of Bivdi's MPL-2.0 code; it is used as a separate component behind published interfaces. (GPLv2 permits sale; the constraint Bivdi respects is source conveyance, not resale.)

---

## 7. Further reading

- [Getting started](docs/getting-started.md) — build, test, and run the Runtime in minutes.
- [seL4](https://sel4.systems) · [WASI](https://wasi.dev) · [Fuchsia](https://fuchsia.dev) · [Genode](https://genode.org) · [Redox](https://www.redox-os.org) · [Qubes OS](https://www.qubes-os.org) · [NixOS](https://nixos.org) · [Firecracker](https://firecracker-microvm.github.io)
- Klein et al., *seL4: Formal Verification of an OS Kernel* (SOSP 2009)
- LeVasseur et al., *Unmodified Device Driver Reuse … via Virtual Machines* (OSDI 2004)
- Waldo et al., *A Note on Distributed Computing* (1994)
- Hardy, *The Confused Deputy* (1988)

---

*Bivdi — nothing has ambient authority. Everything must ask.*
