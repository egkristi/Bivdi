# Bivdi — Durable authority tokens (model)

**Status:** Draft. This document describes the **proposed** model for durable authority tokens. It is *not yet decided* — the exact encoding, field layout, and the boundary with the kernel capability model are **open**, pending `P-001` (kernel) and `P-003` (naming).

> Everything in this document is a *proposal*, not a decision. It is recorded here so the concept has a home, but it carries no normative weight until an RFC is accepted.

---

## 1. Purpose

Kernel capabilities are authority *in use*: they are local, live, and die with their process. Real systems also need authority that is **durable** (persisted) and **delegable** (transferable). The durable token is the proposed answer: authority *at rest and in transit*.

---

## 2. Proposed model

### 2.1 Two tiers

| Tier | Scope | Persistence | Forgery |
|---|---|---|---|
| Kernel capability | Local, live | Dies with its process | Unrepresentable by construction |
| Durable token | Transferable, offline-verifiable | Persisted, transmitted | Cryptographically infeasible |

### 2.2 Attenuation without authority

Any holder may append a restriction (caveat) and re-chain, producing a strictly weaker token, **without contacting an authority**. Removing a caveat requires the root key, which no holder has. Delegation is therefore a local operation.

### 2.3 Proposed caveats

Restrictions that narrow when, where, by whom, or on what a token may be used:

- **expiry** — valid only until a time;
- **not-before** — valid only from a time;
- **bind-component** — holder must be this code (by content hash);
- **bind-attestation** — holder must be in this verified boot state;
- **max-invocations** — a monotonic usage bound;
- **path-prefix** — restricts a catalog subtree;
- **rate-limit** — usage rate bound;
- **audience** — intended verifier.

### 2.4 Revocation (layered)

- **Short default expiries** — minutes to hours, not months.
- **Epoch** — bumping the authority epoch invalidates every token derived from it at once.
- **Revocation set** — explicit revocation of token prefixes.

### 2.5 The warden bridge

A **warden** service holds real kernel capabilities and, after verifying a presented token's chain and caveats, performs the operation with its own capability on the caller's behalf. Tokens never grant kernel authority directly, which keeps cryptography outside the trusted core.

---

## 3. What is certain (decided) vs. proposed

**Certain:** the *concept* that durable, delegable authority is needed, and that it must be attenuable, revocable, time-limited, and attributable (these follow from the decided capability model).

**Proposed (not decided):** the macaroon/HMAC-chain construction, the caveat taxonomy, the CBOR-style encoding, and every specific field layout.

---

## 4. Open questions

1. **Encoding and field layout** — open.
2. **Component naming** (`P-003`) — token names and identifiers use generic terms until decided.
3. **Kernel capability boundary** — how the warden maps tokens to kernel capabilities depends on the kernel choice (`P-001`).
4. **Revocation-set scaling** — under what token volume does the revocation mechanism degrade.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
