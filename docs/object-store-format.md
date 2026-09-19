# Bivdi — Object store (model)

**Status:** Draft. This document describes the **decided** object-store model. The *on-disk encoding, hash algorithm, and serialization format are **open***.

---

## 1. Purpose

The object store replaces the filesystem as the conceptual center of the operating system. Data lives in a typed, content-addressed graph with metadata, relationships, and version history. The filesystem survives as **one view** of the graph — a compatibility view, never the center.

---

## 2. Decided primitives

| Primitive | Description | Property |
|---|---|---|
| **Blob** | An immutable byte sequence named by its content hash. | Identity *is* content; deduplication and integrity verification are inherent. |
| **Cell** | A mutable single-slot holder of a blob reference. | Updated only by compare-and-swap; the *only* mutation primitive. |
| **Catalog** | A persistent map from names to blobs, cells, and sub-catalogs. | Reached **only by capability** — no root, no `/`, no upward walk. |

Two components handed two different catalogs inhabit disjoint universes and cannot discover each other.

---

## 3. Decided properties

### 3.1 Content addressing

Blobs are named by hash. *(The hash algorithm is **open**; BLAKE3 is a leading proposal, not a decision.)*

### 3.2 Deep immutability

Writes are append-only; mutation creates a new node pointing at immutable parents. Rollback to any prior state is a pointer change.

### 3.3 Transactions

All changes are transactions. There is no partially-written state and no `fsck`. A commit is atomic; crash consistency is a property of the design, not of `fsync` discipline.

### 3.4 Snapshots

A snapshot is a retained root — O(1) to take, free to hold. "What did this look like on Tuesday?" is a query, not a restore-from-backup.

### 3.5 Per-object encryption

Each object (or group) is encrypted with its own key, sealed to the boot measurement. Objects can replicate to untrusted storage while leaking only their length.

### 3.6 Crypto-shredding (D-010)

Because history is immutable, deletion is implemented by destroying the key. All copies and history become unreadable at once; "right to be forgotten" is satisfied without rewriting history.

### 3.7 No machine-state persistence (D-007)

The store persists **data and configuration**, not running instruction-level machine state. A reboot yields clean execution state over consistent data; services may checkpoint periodically.

### 3.8 Files survive as contracts

The file is not killed. Objects have a canonical, documented serialized form and can always be exported/imported as files. Legacy programs see objects as files via a virtual filesystem view.

---

## 4. Bounded scope

The store offers transactions, snapshots, indexes, event streams, and replication. It does **not** replace PostgreSQL, Kafka, or large-scale object storage. Databases are applications that use the store's primitives.

---

## 5. Open questions

1. **On-disk encoding** — the copy-on-write layout, extent log, and epoch/root format. *(The Runtime currently persists via deterministic CBOR — `bivdi-object::save_to_path`/`load_from_path` write to a temp file, `fsync`, then atomically rename — as a provisional durable encoding per RFC 0002 §3.2; the copy-on-write on-disk layout and a WAL/crash-recovery are still open.)*
2. **Hash algorithm** — BLAKE3 proposed, not decided.
3. **Encryption specifics** — cipher, key derivation, and per-object granularity.
4. **Schema evolution** — how persistent objects migrate when their types change (`README.md` §17).
5. **Garbage collection** under memory pressure.

Resolution of these must be recorded as an RFC in `rfcs/` and reflected in `README.md` §16–17.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
