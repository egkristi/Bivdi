# Bivdi — Glossary

Terms used across the project documentation. Definitions follow the decided model; where a term depends on an open question, that is noted.

| Term | Meaning |
|---|---|
| **Ambient authority** | Power a program holds by virtue of *who* runs it rather than *what* it was given. Bivdi has none. |
| **Attenuation** | Deriving a strictly weaker capability or token from a stronger one. Always permitted; never reversible. |
| **Capability** | An unforgeable handle that both points to a resource and grants a specific right to it. |
| **Capability runtime** | The service that mints, attenuates, leases, and revokes capabilities, and records provenance. |
| **Confused deputy** | A privileged component tricked into misusing its authority. Structurally prevented by making designation imply authority. |
| **Crypto-shredding** | Deleting data by destroying its encryption key, so all copies and history become unreadable at once. |
| **Declarative state** | Describing *what should be* rather than imperatively issuing commands; the system reconciles toward it. |
| **Desired state** | The declarative specification of a system's intended configuration and workloads. |
| **Driver VM** | A deprivileged virtual machine that owns a hardware device and exposes it onward, isolated behind an IOMMU. |
| **Event bus** | The fabric carrying typed, first-class records of change. |
| **Generation** | An immutable, complete system state that can be activated or rolled back transactionally. |
| **IDL** | Interface Definition Language — a language-neutral way to describe protocols so bindings can be generated. |
| **IOMMU** | Hardware that restricts which memory a device can reach via DMA. Required by Bivdi. |
| **Lease** | A time-limited or scope-limited permission ("microphone for 5 minutes", "until this task ends"). |
| **Object store** | The typed, content-addressed, versioned storage that replaces the filesystem as the conceptual center. |
| **Petname** | A local, user-assigned name for a cryptographically identified party; shown in place of self-claimed names. |
| **Powerbox** | A trusted, unspoofable UI through which a user gesture (e.g., picking a file) *is* the capability grant. |
| **Provenance** | The attributable record of who wrote what, with which capability chain, from which code. |
| **Reconciliation** | Continuously bringing observed state toward desired state. |
| **State engine** | The service that accepts desired state, computes diffs, and activates generations atomically. |
| **TCB** | Trusted computing base — the code whose failure breaks the security model. |
| **WASI** | WebAssembly System Interface — the capability-oriented system interface for WebAssembly. |
| **Workload** | The execution unit: immutable code + an explicit capability set + a resource budget. |

---

## Deferred naming

The core components are currently referred to by generic terms ("object store", "capability runtime", "state engine", "event bus", "identity service", "agent host"). Final names are an open decision (`P-003`). No codenames or daemon-style abbreviations should appear in code or docs until a naming decision is recorded.
