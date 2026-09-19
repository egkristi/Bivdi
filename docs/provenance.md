# Bivdi — Provenance and Audit

**Status:** Draft, grounded in the decided model. Provenance is a load-bearing commitment. The concrete log encoding is **open**; the *model* below is decided.

> **Implementation gap (2026-09-20 audit, H4).** The Runtime's provenance is currently an in-memory `Vec<Event>` with a push: no hash chaining, no Merkle tree, no signature, no durability. The integrity properties in §4 describe the *decided model*, not what the Runtime implements today. Closing this gap is sequenced in `ROADMAP.md` (Milestone A remaining work, item 6): append-only and hash-chained first, then durability, then signed checkpoints and replication.

---

## 1. Decided principle

Every write is attributable. "**What changed my machine, and what gave it the right to?**" is a query, not a forensic investigation.

---

## 2. The provenance log

An append-only, hash-chained log records every **authority-relevant** event:

- component instantiation, with manifest digest;
- every capability mint, copy, move, and revoke, with parent linkage;
- every durable-token issuance, verification, and rejection;
- every object mutation, with before/after content hashes;
- every driver IOMMU-domain change;
- every update transaction;
- every boot, with its measurement set.

---

## 3. Authority, never content

The log records **authority, never content** — no data contents, no keystrokes, no network payloads. This distinction is load-bearing: an audit log that captures data becomes the most valuable target on the system.

---

## 4. Integrity properties

- **Append-only** and **hash-chained** (each entry commits to the previous).
- A Merkle tree over each epoch; epoch roots are signed.
- An attacker who fully compromises the machine still cannot rewrite history undetected: deletion is visible as a gap, and forking as a root mismatch.
- Optionally co-signed to an external transparency log. *(Specific signing/transparency mechanisms are open.)*

---

## 5. Query model

The log answers structured questions over a capability-restricted view:

- *Why* was this object modified?
- *Which capability chain* authorized it?
- *Which code* (by content hash) performed it?

A query returns the complete authority chain back to the root, the reason the operation was permitted, and the content hash of the code that did it — never a mutable name.

---

## 6. Open questions

- Concrete log encoding, signing keys, and transparency-log integration.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
