# Bivdi — Performance

**Status:** Draft. Performance is a first-class attribute (`D-011`). The *reference targets* below are **proposed** (to be ratified in spec v0.1 and turned into CI gates); the *principle* is decided.

---

## 1. Decided principle

Performance is a first-class, measurable attribute — designed in from the start and gated in CI, not optimized after the fact. It is **never won by weakening the security model**: isolation has an accepted, budgeted cost, and the goal is predictability within that budget rather than peak throughput at any price.

---

## 2. Reference targets (proposed)

Measured on the reference platforms. Once ratified, regressions past these fail CI.

| Metric | Target | Notes |
|---|---|---|
| IPC round trip (same core, ~4 words) | < 500 cycles | The microkernel cost center |
| IPC round trip (cross-core) | < 2,000 cycles | |
| Interrupt → userspace driver thread | < 3 µs | Adequate for realtime control |
| Component instantiation (cold) | < 2 ms | Restart must feel instant |
| Driver restart after fault | < 10 ms | Below human perception |
| Boot: firmware → login | < 400 ms | |
| Kernel static memory | < 1 MiB | |
| Minimum useful system (kernel + object store + shell) | < 32 MiB RAM | |
| Object-store sequential read | ≥ 85% of raw device | Storage overhead must be modest |
| Object-store 4K random IOPS | ≥ 70% of raw device | |
| Durable-token verification (5 caveats) | < 20 µs | Cheap enough to do per request |
| Syscall-heavy workload vs. a monolithic kernel | within 30% | The accepted isolation tax |

---

## 3. The framing

The last row is the point: Bivdi accepts a measurable cost for isolation, and staying inside that budget is a hard requirement — not an apology, and not a trade-off against security.

Performance-sensitive paths are IPC, the object store, and scheduling. These carry the regression gates.

---

## 4. Open questions

- Exact target values are **proposed**; ratification belongs to spec v0.1.
- GPU acceleration vs. a small TCB may impose a performance cost that is as yet unquantified (`README.md` §17).

---

*Bivdi — nothing has ambient authority. Everything must ask.*
