# Bivdi — Compatibility

**Status:** Draft, grounded in the decided model (`README.md` §16). This document elaborates how existing software runs on Bivdi without weakening the native model. It asserts no not-yet-decided specifics.

---

## 1. Decided principle

**Compatibility without compromise** (`README.md` §3, core invariant): legacy software runs in isolated subsystems and never weakens the native capability model.

Compatibility is a *first-class subsystem*, not an afterthought. But it is *emulation*, not compromise — a legacy program never receives authority beyond what it was handed, and it can never escape its isolation into the native system.

---

## 2. The four levels

| Level | Mechanism | Compatibility | Integration |
|---|---|---|---|
| **1. Native** | WASI components and native Bivdi programs | New software | Full: capabilities, objects, events |
| **2. Linux ABI** | User-space syscall translation (Starnix pattern) | High for CLI/server software | Sees objects as files, bounded by capabilities |
| **3. Micro-VM** | Isolated Linux/Windows kernel | Very high | Via virtio and proxies |
| **4. Recompile** | POSIX library (relibc-style) | Source-available software | Medium |

---

## 3. Level 1 — Native

WASI components are the first-class native application format (`D-004`). They are capability-oriented by design, and the component model provides typed, language-neutral interfaces. Rust, C, C++, Go, Swift, C#, and others already target WASI.

WASI is first-class but **not exclusive**: native programs remain necessary for maximum performance, hardware access, and systems software.

---

## 4. Level 2 — Linux ABI (user-space translation)

Unmodified Linux binaries run through a user-space translation layer that implements Linux syscalls (the Starnix pattern).

- Each Linux process's filesystem view is a catalog it was handed; `/` is *its* root and nothing more. `open("/etc/passwd")` succeeds only if the granted catalog contains that entry.
- Network syscalls map to flow capabilities; a Linux process gets exactly the egress its Bivdi manifest granted.
- A Linux process cannot `ptrace` outside its own tree, cannot load kernel modules, and cannot obtain `CAP_SYS_ADMIN`-class authority — those have no counterpart in a system without ambient authority.

**Honest limits.** The Linux ABI is not just ~400 syscalls; it is also procfs, sysfs, netlink, ioctl, cgroups, and eBPF. It is deliberately incomplete and will never be bug-compatible. It targets CLI tooling, language runtimes, compilers, and headless services — not desktop applications expecting DBus/systemd.

---

## 5. Level 3 — Micro-VM

Heavyweight legacy workloads (a real Linux or Windows kernel) run in an isolated micro-VM. This is the fallback for software that genuinely needs a full kernel.

- Windows programs run via a Wine-like environment inside the Linux layer or a micro-VM.
- The first browser on Bivdi runs in level 2 or 3; a native browser is a multi-year project.

---

## 6. Level 4 — Recompile

For software with source, a POSIX library (relibc-style) provides a cheap porting path. Recompiling to WASI is the main route to native.

---

## 7. Cannibalization risk (acknowledged, decided response)

A too-good compatibility layer risks becoming the only thing developers target (the OS/2 effect). Bivdi's answer is **not** to make legacy software deliberately worse — the most important programs will be legacy for years. Instead:

1. **Cheap porting** — recompiling to WASI is the main path to native.
2. **Native advantages** — delegated agent capabilities, the object graph, provenance, generations, and cross-device sharing are unavailable to legacy software.
3. **Greenfield starting areas** — agent tooling and new services are written native from the start.

---

## 8. Open questions (from `README.md` §17)

- **Schema evolution** — affects the files-as-contracts boundary.
- **GUI in WASI** — there is no mature standard for windows/GPU yet; wait, contribute, or define.
- **Driver availability** — the existential risk behind level-3 reliance.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
