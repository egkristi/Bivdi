# RFC 0002 — Interface definition language and wire format

**Status:** Accepted
**Issue:** #39
**Resolves:** "IDL choice and wire format" (`README.md` §17, `docs/abi.md` §5, `docs/spec.md` §14.1)
**Date:** 2026-09-19

---

## 1. Summary

Adopt **WIT** (the WebAssembly Component Model's interface definition language) as Bivdi's IDL, with the **Component Model canonical ABI** as the in-process wire format and a **CBOR-encoded message form** for the same types when they cross a process, machine, or persistence boundary. Generate bindings rather than hand-writing them.

## 2. Motivation

The IDL is named as a Phase 0 deliverable, and `docs/abi.md` states outright that it is blocked on this choice. It is the largest piece of unblocked work in the project: none of it depends on `P-001`.

It is also the decision with the longest half-life. `D-003` makes one interface contract the thing that lets the Runtime today and any future Core later be the same system, and `D-005` makes the wire format — not a language calling convention — the contract. Every service written before the IDL exists is written against an implicit interface that will have to be restated later; six crates have already been written that way.

## 3. Proposal

### 3.1 The IDL is WIT

`D-004` already decides that WASI components are the native application format. WIT is the interface language those components are defined in. Choosing anything else means every native application interface is described twice — once in Bivdi's IDL and once in WIT — with a translation layer between them that can drift.

WIT satisfies the requirements `docs/abi.md` §3 sets out:

| Requirement | WIT |
|---|---|
| Objects — typed values with identity | records, variants, and `resource` types |
| Capabilities — transfer and attenuation | `resource` handles with move semantics; **own** and **borrow** are in the type system |
| Typed calls with structured results | functions returning `result<T, E>` |
| Events | `resource` streams, or an explicit event interface |
| Streams, bounded and backpressured | `stream<T>` in the async/`wasi:io` model |
| Cancellation | `pollable`, and explicit drop of a handle |
| Structured errors | `result<_, error-variant>`, never a string code |
| Deadlines and priorities | carried as parameters; not part of the type system |

The `own`/`borrow` distinction deserves emphasis: it is the closest thing in any mainstream IDL to the ownership-transfer rule in `docs/abi.md` §2.6 — a buffer passed as a capability with the sender losing access in the same instant. In WIT that is `own<T>` and it is checked at the binding boundary rather than documented in prose.

### 3.2 The wire format is layered, deliberately

One IDL, two encodings, chosen by boundary:

- **In-process and component-to-component:** the Component Model **canonical ABI**. It is the format WASI components already speak, so a native Bivdi application talks to a Bivdi service with no translation at all.
- **Across a process, a machine, or into storage:** **CBOR** (RFC 8949), with a deterministic encoding profile, generated from the same WIT types.

The second encoding exists because the canonical ABI is a linear-memory calling convention, not a serialisation format: it is not self-describing, not versioned, and not something to write into a durable object. CBOR is self-describing, has a defined deterministic form suitable for content addressing and hash-chained logs, is compact, and has mature implementations in Rust.

The rule is: **the canonical ABI is how components call each other; CBOR is how Bivdi remembers and transmits.** A service defines one WIT interface and gets both.

### 3.3 Generated, never hand-written

Bindings are generated for Rust first (`wit-bindgen`), then C, then whatever the SDK demands. No hand-written ABI calls, in accordance with `docs/abi.md` §2.1.

### 3.4 Versioning and evolution

- WIT packages carry semantic versions; interfaces are versioned as `bivdi:capability@0.1.0`.
- Evolution rules follow the Component Model's: adding an optional field or a new variant case is compatible; removing or reordering is not.
- The deterministic CBOR profile pins map key ordering and integer encoding so that the same logical value always hashes identically — required by content addressing and by the hash-chained provenance log.

Schema evolution for *persistent objects* is a separate and harder question (`README.md` §17 item 2) and is **not** resolved here. This RFC resolves interface evolution only.

### 3.5 Kernel independence of the IDL

The IDL must not acquire any concept that binds it to a particular kernel. **No seL4 concept enters the IDL.** Capabilities, objects, events, and state are expressed in WIT terms (resources, records, variants, streams) — never as seL4 cnode/capability-object terminology, badge bits, or invocation numbers. The microkernel decision (`P-001`) is deferred, and the IDL is the load-bearing contract that keeps the door open: whatever kernel is eventually chosen, the interfaces Bivdi's own components speak must be restatable unchanged.

This rule is what makes the Runtime (today) and any future Core (later) the *same system*: the contract is the WIT interface, and the kernel is an implementation of the contract, not a vocabulary the contract borrows.

### 3.6 What this does to the existing runtime

The six crates currently expose Rust APIs with no interface definition behind them. The migration is to write the WIT first and make the Rust conform, not to retrofit generated code:

```wit
package bivdi:core@0.1.0;

interface capability {
    enum right { read, write, execute, grant, signal, revoke }
    resource cap {
        attenuate: func(to: right) -> result<cap, cap-error>;
        rights: func() -> right;
    }
    ...
}
```

Writing that interface immediately surfaces a design problem the Rust has been hiding, which is a point in its favour — see §5.

## 4. Alternatives considered

**Cap'n Proto.** Genuinely excellent, and its capability model — with promise pipelining and a level-3 network protocol — is closer to Bivdi's semantics than anything else on this list. Rejected as the *primary* IDL for one reason: it would sit alongside WIT rather than replace it, because `D-004` means WASI components must be describable regardless. Two IDLs is the outcome this RFC exists to avoid. Cap'n Proto's promise pipelining remains worth revisiting for the distributed path in Phase 2, and nothing here forecloses it.

**FIDL (Fuchsia).** Well-suited to exactly this problem and battle-tested at scale, with a strong handle-passing model. Rejected because it is developed for and coupled to Fuchsia, has no WASI story, and adopting it means maintaining a fork or tracking an upstream whose priorities are someone else's.

**Protocol Buffers / gRPC.** Rejected: no capability or handle concept at all, no ownership transfer, and a wire format that encourages string-keyed, schema-optional evolution — the opposite of a typed contract. Retrofitting handles onto protobuf produces the ambient-authority patterns Bivdi exists to remove.

**A bespoke Bivdi IDL.** Maximum fit, and the wrong use of the project's scarcest resource. It also loses the thing that makes WIT compelling: existing toolchains in Rust, C, C++, Go, Swift and C# already target it, which is most of `D-005`'s binding-target list for free.

**CBOR everywhere, including in-process.** Simpler to describe, and it would make every component-to-component call pay serialisation cost against the "within 30% of a monolithic kernel" target in `ARCHITECTURE.md` §20. Rejected on performance grounds.

## 5. Security analysis

The IDL is the enforcement surface for two decided invariants, and choosing one makes both checkable rather than aspirational.

**Capability-aware interfaces.** `docs/abi.md` §2.3 requires that an operation be reachable only through a capability that grants it. In WIT, a `resource` handle is the only way to name an instance, and handles cannot be forged or fabricated by a component — the host mediates the handle table. That is `no ambient authority` expressed in the type system rather than in review comments.

**Ownership transfer.** `own<T>` makes the move-semantics rule in `docs/abi.md` §2.6 a compile-time property at the binding boundary.

**This choice exposes an existing defect.** Writing the capability interface in WIT forces the `right` type to be declared, and `ARCHITECTURE.md` §4.2 names six rights — `read`, `write`, `execute`, `grant`, `signal`, `revoke` — which do not form a total order. The runtime currently implements `Right` as a three-value ordered enum with attenuation as `new <= held`. `execute` is not "less than" `write`, and a flag set is not a chain. **The IDL should declare `flags right { read, write, execute, grant, signal, revoke }` and attenuation should be subset inclusion.** This is a smaller change now than at any later point, and it is a prerequisite for writing the interface honestly.

**Deterministic encoding is a security property, not an optimisation.** The provenance log is hash-chained and the object store is content-addressed. A non-deterministic encoding means the same logical value produces different hashes, which breaks deduplication and makes log verification unreliable. The CBOR profile must pin this, and the conformance suite must test it.

No authority is widened by this RFC.

## 6. TCB impact

**Neutral to negative, which is the right direction.** The IDL and its generated bindings live entirely in userspace. The kernel never parses a wire format — `ARCHITECTURE.md` §14.2 states it never parses a string, and nothing here changes that.

One caution to record: the canonical ABI is implemented by the component host, so the host's handle-table implementation becomes correctness-critical for capability confinement on the Runtime track. It is not in the TCB as defined (it is not the kernel, the supervisor, or the root authority), but it is the component whose failure would most resemble a TCB failure on the Linux track, and it should be called out in `docs/threat-model.md` as such.

## 7. Performance impact

The canonical ABI is designed for low-overhead component-to-component calls and is the right tool against the sub-500-cycle IPC target — though that target is a *kernel* IPC number and this is a layer above it.

CBOR encoding costs apply only at boundaries that were already paying serialisation cost. Deterministic CBOR is modestly slower than a permissive encoder because it sorts map keys; that cost lands on persistence and network paths, not on the IPC fast path.

One target needs re-examination once this lands: **component instantiation under 2 ms cold** is optimistic for WASI components unless they are ahead-of-time compiled and cached. That should be stated in `docs/performance.md` rather than discovered on the first benchmark.

## 8. Migration plan

1. **Define `bivdi:core@0.1.0` in WIT** covering the interfaces the six crates already imply: capability, object store, state engine, event bus, identity, agent host.
2. **Fix the rights lattice first** (§5) — `flags` rather than an ordered enum — because the interface cannot be written truthfully otherwise. This is a breaking change to `bivdi-cap` and should land before more code depends on the ordering.
3. **Generate Rust bindings** and make each crate implement its generated trait, keeping the current hand-written API as a thin façade until callers migrate.
4. **Add the deterministic CBOR profile** and use it for the provenance log and blob encoding.
5. **Write a conformance suite** from the WIT: the same test vectors must pass against Bivdi Runtime today and any Bivdi Core later. This suite *is* the "one contract" guarantee in `D-003` — without it, that guarantee is an intention.

Steps 1 and 2 are the Phase 0 exit-gate work. Steps 3 to 5 can proceed in parallel with any kernel decision.

## 9. Document updates

On acceptance:

- `README.md` §16 — add as `D-015`; §17 item 1 (IDL choice) struck.
- `docs/decisions.md` — add `D-015` with a pointer to this RFC; note the kernel-independence rule (§3.5) as the mechanism that keeps `P-001` open.
- `docs/abi.md` — §5 open questions replaced by the decided IDL and encodings; the document is unblocked.
- `docs/spec.md` §7 and §14 — IDL named; open-question list shortened.
- `docs/object-store-format.md` — reference the deterministic CBOR profile for the on-disk encoding question.
- `docs/threat-model.md` — add the component host's handle table as correctness-critical on the Runtime track.
- `docs/performance.md` — qualify the component-instantiation target for AOT-compiled components.
- `ROADMAP.md` — "ABI/IDL definition" becomes tractable; add the conformance suite as a milestone condition.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
