# Bivdi — Threat Model

**Status:** Working draft, grounded in the decided security model (`README.md` §16). This document grows as the architecture does; it is updated alongside any architectural change.

**Scope.** This threat model describes the *decided* security commitments and the adversaries they address. Details that depend on open questions (kernel choice, component naming, specific token/object formats) are marked *proposed*.

---

## 1. Purpose and scope

Bivdi's security goal is structural: **a compromised or malicious component can only exercise the authority it was explicitly given.** The threat model enumerates who is being defended against, what they can do, and what the system's answers are.

---

## 2. Assets

| Asset | Why it matters |
|---|---|
| **Data objects** | User and organization data in the object store |
| **Capabilities** | The authority to act on objects, services, and devices |
| **Identities and keys** | Cryptographic identity underpinning nodes, services, and delegation |
| **System state** | Desired state, generations, and configuration |
| **Provenance/audit log** | The record of who did what, with which authority |
| **Compute resources** | CPU, memory, GPU/NPU, storage, network, devices |

---

## 3. Trusted computing base (TCB)

The TCB is the minimum set whose failure breaks the security model. Under the decided model it consists of:

1. the CPU and its IOMMU;
2. the measured boot chain up to and including the kernel;
3. the kernel (choice **open**; seL4 proposed);
4. the root supervisor;
5. the root authority holding the seal/recovery key.

**Everything else is outside the TCB** — the object store, every driver, the network stack, the compositor, and every application. A defect in any of them is a bug, not a breach of the model.

> The exact TCB list is subject to the kernel decision (`P-001`). This list is the invariant to preserve under any kernel choice.

---

## 4. Adversaries and mitigations

| Adversary | Assumed capability | Bivdi's answer |
|---|---|---|
| **Malicious application** | Arbitrary code in its own isolation domain | Holds only manifest-granted capabilities; cannot name other components' resources |
| **Compromised dependency** | Arbitrary code inside an otherwise-trusted component | Blast radius limited to that component's capability set; provenance records the code hash |
| **Compromised driver** | Arbitrary code in a userspace driver with device access | Confined to its IOMMU domain and capability set; cannot touch kernel memory or other drivers; restartable |
| **Malicious peripheral (DMA)** | Bus-master DMA, malicious USB/Thunderbolt | IOMMU with per-buffer pages and strict invalidation; no driver gets an identity-mapped window |
| **Network attacker** | Full control of the path, MITM | Encryption and mutual authentication by default; flow capabilities, not raw sockets |
| **Prompt-injection / compromised agent** | An agent tricked into misusing its delegated authority | Narrow, attenuated, time-limited, quota-bound capabilities; content is data, not instructions; full provenance |
| **Supply-chain attack** | A backdoored package or update | Content-addressed, reproducible builds; authority-widening changes surface as a readable diff |
| **Local privilege-escalation seeker** | Syscall fuzzing, race exploitation | Small capability-based syscall surface; no name-based access, so no TOCTOU-on-name class |
| **Offline disk attacker** | Physical possession of powered-off storage | Per-object encryption sealed to the boot measurement |
| **Insider with a valid credential** | A legitimate capability/token, misused | Attenuated tokens, short expiries, complete provenance, instant revocation |
| **Ransomware** | A component that can write, misusing it | No ambient write access; versioned object store makes rollback a pointer change |
| **Lost device** | Physical loss of a powered-off device | Data encrypted at rest; root capability in hardware; recovery via identity |

---

## 5. Out of scope (v1)

- **Malicious silicon / hardware backdoors.** The CPU and IOMMU are trusted to implement their interfaces.
- **Novel speculative-execution side channels.** Known mitigations are deployed where the hardware allows; resistance to the next Spectre-class variant is not claimed.
- **Physical attack on a running machine** (cold boot, bus probing, chip decapping, evil-maid against a powered system).
- **Traffic analysis.** Bivdi hides *what* was communicated and by whom, not *that* communication occurred.
- **Denial of service by an authorized holder.** A component granted a CPU reservation may waste it.
- **The user's own decisions.** If a human grants broad authority through a legitimate gesture, Bivdi enforces it faithfully.

---

## 6. Explicit trust assumptions

- The CPU implements its ISA; the IOMMU enforces DMA isolation.
- The boot chain (firmware → boot loader → kernel) is the root of trust and is measured.
- The root authority (seal/recovery key) is held in hardware and used only by explicit local action.

---

## 7. Open threat-model questions

1. **Revocation of shared memory and copied-out data** — unsolved in the general case; mitigation strategy pending.
2. **GPU acceleration vs. isolation** — modern GPU drivers want broad DMA; there may be no way to have both hardware-accelerated graphics and a small TCB.
3. **Driver availability vs. assurance** — hold a narrow hardware list, or accept a Linux driver shim with its assurance cost.
4. **Semantic indexing without a broad-access indexer** — search over all data conflicts with least privilege; the resolution is open.
