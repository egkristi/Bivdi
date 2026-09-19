# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Identity-based networking and flow capabilities** (`bivdi-net`) — identity-based endpoints, flow capabilities instead of raw sockets, DNS-as-capability resolution, and structural absence of network access without a flow.
- **Queryable provenance** — `Node::provenance_query()` returns authority-relevant events (granted/used/revoked) in append order, satisfying the "provenance is queryable, not merely recorded" requirement.
- **WASI runtime with least-authority capability scoping** (`bivdi-wasm`) — executes WASI Preview 1 command modules (`_start`) and typed exported functions with an explicit capability context (stdout only by default; no filesystem/network unless granted).
- **Integrated runtime node** (`bivdi-runtime`) — composes the six primitives into one running system, with authority events flowing onto the event fabric; includes the RFC 0001 §3.4 agent-isolation scenario.
- **Identity service** (`bivdi-identity`) — identity kinds, cryptographic fingerprints, petnames, and selective disclosure.
- **Event bus** (`bivdi-event`) — typed, first-class events with correlation identity, filtered pub/sub, and append-only history.
- **Agent host** (`bivdi-agent`) — capability-constrained, time-limited, quota-bound agents; delegation cannot escalate.
- **Bivdi Runtime** (Phase 0) — object store (`bivdi-object`, content-addressed blobs, CAS cells, catalogs), capability runtime (`bivdi-cap`, mint/attenuate/revoke/leases/provenance), and state engine (`bivdi-state`, desired state, generations, rollback).
- **Object-store persistence** — save/load with a provisional JSON format; CLI `persist` subcommand.
- **Container-friendly runtime** (`Containerfile`) and `.dockerignore`.
- **CI workflows** — `docs.yml` (required files + internal link validation) and `rust.yml` (fmt, build, clippy `-D warnings`, test).
- **RFCs** — `0001` (target niche), `0002` (IDL), `0003` (platform guarantees), `0004` (kernel choice).
- **Documentation** — full `docs/` set (threat model, spec, ABI, capabilities, state, drivers, networking, AI agents, identity, events, provenance, resources, observability, performance, recovery, compatibility, boot, glossary, decisions) plus `ARCHITECTURE.md`, `ROADMAP.md`, `LICENSING.md`, `SECURITY.md`, `CONTRIBUTING.md`, and `AGENTS.md`.
- **`LICENSE`** (MPL-2.0).

### Changed
- **Removed name-verification content** — the name is settled and usable; all orthography/language-authority/rename/trademark-search text was removed.
- **Refreshed status wording** across `README.md`, `runtime/README.md`, `SECURITY.md`, `CONTRIBUTING.md`, and `ROADMAP.md` to reflect the implemented runtime prototype (which is *not* an operating system).
- **MSRV and toolchain raised 1.75 → 1.85 → 1.88** to match actual dependency requirements.

### Fixed
- **Authority-escalation bugs** — `spawn` no longer mints authority from nothing; delegation clamps time and quota; state-engine rollback links to the correct predecessor and the lock-ordering inversion was removed.
- **Plan enforcement** — `execute_plan` now enforces per-step rights instead of a hard-coded `Read`.

### Security
- **No ambient authority** at every boundary: capability runtime, agent host, and WASI runtime all enforce least-authority by construction.
