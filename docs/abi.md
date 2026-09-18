# Bivdi — ABI and interface requirements

**Status:** Draft. This document states the **decided** requirements for the ABI and the interface-definition approach. The *specific IDL and its encoding are **open*** — this document defines *what the ABI must do*, not *which IDL*.

---

## 1. Purpose

The ABI is the contract between components. It is the one thing both implementation tracks (`Bivdi Runtime` on Linux, `Bivdi Core` on a microkernel) share, so a program written against it runs on either unchanged (`D-003`).

---

## 2. Decided requirements

### 2.1 Language-neutral IDL

Component interfaces are defined in a **language-neutral IDL** (`D-005`). Bindings are **generated** for Rust, C, C++, Go, Swift, and WASI components — not hand-written ABI calls.

### 2.2 The wire format is the contract

The delivery/encoding format is the contract, **not** a language calling convention. This has two consequences:

- The interface is stable across language updates and across the two tracks.
- Rust's lack of a stable ABI is irrelevant: Rust is a language for *implementing* components, never a binding target that others must call into.

### 2.3 Capability-aware

Every interface boundary is capability-aware: an operation is only reachable through a capability that grants it, and a capability can be transferred across an interface. There is no global namespace through which a component could name a service it was not handed.

### 2.4 Typed, not textual

Protocols are typed. Protocol violations are caught at compile time wherever possible, not in production. Text is a presentation format, not an integration format.

### 2.5 Asynchronous by default, with a fast sync path

Messages are asynchronous with promises. Synchronous calls exist for short, local operations where the fast path gives low latency.

### 2.6 Large data by ownership transfer

Bulk transfer (graphics, ML, video) uses **ownership transfer**: a buffer is passed as a capability, and the sender loses access in the same instant — move semantics across trust boundaries. Shared, simultaneously mutable memory across trust boundaries is not allowed (except explicitly agreed ring buffers between a driver and its service).

---

## 3. What an interface must be able to express

A conforming interface must support, at minimum:

- **objects** (typed values with identity),
- **capabilities** (transfer and attenuation),
- **typed calls** (request/response with structured results),
- **events** (typed, first-class),
- **streams** (bounded, backpressured),
- **cancellation** (explicit, propagatable),
- **transactions** (for the object store and state engine),
- **structured errors** (not string error codes),
- **deadlines and priorities** (for scheduling-sensitive calls).

---

## 4. Binding targets

Decided (`D-004`, `D-005`):

- **Rust** (services, drivers, kernel).
- **C and C++** (via a C ABI).
- **Go, Swift** (service and tooling ecosystems).
- **WASI components** (the native application format).

---

## 5. Open questions

1. **Which IDL?** Options include the FIDL/Cap'n Proto style (typed, versioned schemas, generated bindings). No choice is decided.
2. **Encoding** — canonical binary representation, versioning and evolution rules, and back-compat guarantees.
3. **Schema evolution semantics** — how interfaces evolve without breaking existing components (this is a system-wide open question, `README.md` §17).

Resolution of the IDL choice must be recorded as an RFC in `rfcs/` and reflected in `README.md` §16–17.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
