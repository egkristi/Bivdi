# Bivdi — Capabilities

**Status:** Draft, grounded in the decided model. The capability model is the heart of Bivdi's security; this document states it authoritatively. Where detail is proposed (durable tokens, kernel mapping), it is marked.

---

## 1. Decided principle

**No ambient authority.** A component's authority is exactly the capabilities it was handed. There is no `root`, no `sudo`, no global namespace, and no capability obtainable by asking the kernel nicely.

A **capability** is an unforgeable handle that both **points to** a resource and **grants a right** to it.

---

## 2. Properties

- **Unforgeable.** Forgery is *unrepresentable*, not merely hard — a capability is validated by the kernel on every use, not a bit pattern a holder can inspect or manufacture.
- **Transferable.** A holder may pass a capability on, but only if it holds it.
- **Attenuable.** A holder may derive a strictly weaker capability (read-only, one subtree, one deadline). Attenuation never requires authority and is never reversible.
- **Revocable.** The granter may invalidate a capability and its entire derived subtree.
- **Scoped and delegatable.** Capabilities are narrowed and handed onward; leases are the default shape of a grant.

---

## 3. Two tiers of authority

| Tier | Scope | Persistence | Forgery |
|---|---|---|---|
| **Kernel capability** | Local, live | Dies with its process | Unrepresentable |
| **Durable token** | Transferable, offline-verifiable | Persisted, transmitted | Cryptographically infeasible |

Kernel capabilities are authority *in use*; durable tokens are authority *at rest and in transit*. *(The durable-token design is **proposed**; see `token-format.md`.)* A **warden** service bridges the two, so cryptography stays outside the trusted core.

---

## 4. No root, but a root capability

"No root" means the authority is not *ambient* — it does not mean there is no highest authority. A **root capability** per machine and identity lives in hardware, is never granted to programs or agents, and is used only for recovery, key rotation, and ownership change.

---

## 5. The powerbox

The user gesture *is* the grant. Selecting an object in a trusted, system-drawn dialog grants the application a capability to exactly the chosen object — and nothing else. This makes least authority usable rather than merely correct.

---

## 6. Leases

Permissions are leases, not permanent flags: "microphone for 5 minutes", "until this task ends". Leases prevent permission hoarding over time.

---

## 7. Open questions

- Revocation of shared memory and of data already copied out of a component.
- The capability runtime's attenuation/revocation model at the Runtime level.
- The durable-token design (proposed).

---

*Bivdi — nothing has ambient authority. Everything must ask.*
