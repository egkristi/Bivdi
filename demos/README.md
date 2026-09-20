# Bivdi demos

Recorded **asciinema v2** terminal sessions showcasing what the Bivdi Runtime
currently does. Every recording is generated from the *actual* output of
`bivdi-cli` — nothing is hand-authored or staged.

## Viewing

Install `asciinema` and play any `.cast` file:

```sh
asciinema play demos/01-six-primitives.cast
```

Or paste the file URL/contents at [asciinema.org](https://asciinema.org/) to
share it.

## The demos

| File | What it shows |
|---|---|
| `01-six-primitives.cast` | The full demo: object store (content addressing + CAS), capability runtime (attenuate/revoke/lease), state engine (generations/rollback), agent host, event bus, identity. |
| `02-agent-scenario.cast` | The RFC 0001 §3.4 exit-gate: a leased Write agent writes, a prompt-injection "read the mailbox" is denied, and the lease expiry blocks the legitimate write — with the detailed authority provenance. |
| `03-persistence.cast` | Object store saved to disk (deterministic CBOR, atomic + `fsync`) and loaded back. |
| `04-wasm.cast` | The WASI host: compile a module, call an exported function, run a WASI command with least authority. |
| `05-sandbox.cast` | seccomp/Landlock state — reported honestly (Landlock is `unavailable` on hosts/containers that forbid it). |

## Regenerating

The recordings are produced by [`generate.py`](generate.py), which runs the
real binary and captures its output verbatim:

```sh
cd runtime && cargo build -p bivdi-cli
cd .. && python3 demos/generate.py
```

> The `timestamp` in each `.cast` header is when it was generated, so
> regenerating produces byte-different files; the *content* is deterministic.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
