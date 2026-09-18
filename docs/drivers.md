# Bivdi — Drivers and Hardware

**Status:** Draft, grounded in the decided model. The VM-first driver target (`D-002`), IOMMU requirement (`D-006`), and driver-VM strategy (`D-009`) are decided.

---

## 1. Decided principle

Driver support is historically where new operating systems die. Bivdi's strategy is layered, with native drivers for standardized devices and driver VMs for everything else.

Every driver is an ordinary, **unprivileged**, **restartable** component — the kernel contains **no device drivers at all**.

---

## 2. The strategy (decided)

1. **Native drivers** for standardized device classes (virtio, NVMe, AHCI, xHCI, a few NIC families) — small, well-documented, written in Rust where possible.
2. **Driver VMs** for everything else (GPU, Wi-Fi, and other difficult hardware) — a deprivileged Linux VM behind IOMMU, exposed via virtio or dedicated protocols.
3. **Native drivers gradually take over**, one device class at a time.

---

## 3. VM-first target (D-002)

Bivdi gets up and running **in virtual machines first**, targeting the most common VM engines and hypervisors:

- **KVM / QEMU**, **VirtualBox**, **VMware**, **Firecracker**, **Proxmox**
- **Public clouds:** AWS (ENA/NVMe), Google Cloud (gVNIC/NVMe), Azure (MANA/NetVSC/NVMe)

`virtio` is the common denominator. VM-first gives a short, well-understood driver list and avoids the hardest desktop-hardware problems (GPU, Wi-Fi, suspend).

---

## 4. Hardware tiers

| Tier | Scope | Status |
|---|---|---|
| **1: Virtualized** | KVM, VirtualBox, VMware, Firecracker, Proxmox, AWS, GCP, Azure | First target |
| **2: Reference servers** | One or two chosen x86-64 / ARM server platforms | Second target |
| **3: Reference laptop** | One chosen laptop (e.g., Framework) | Later phase |
| **4: Arbitrary hardware** | Via driver VM where IOMMU exists | Best effort |
| **—: No IOMMU** | — | Never supported |

---

## 5. Driver isolation (decided)

A driver receives:

- MMIO regions (mapped device-uncached),
- an interrupt handler for its lines,
- an IOMMU domain restricted to its explicitly registered DMA buffers,
- a scheduling context sized for its latency requirement.

It receives **no** ability to touch memory outside its domain — a compromised GPU driver is a graphics outage, not a root compromise. Every DMA buffer must be registered before use, and registration requires holding a capability to that memory.

---

## 6. IOMMU is required (D-006)

Hardware without an IOMMU is unsupported. A device can only DMA into buffers it was explicitly given. IOMMU is necessary but not sufficient (Thunderclap, 2019): Bivdi uses per-buffer pages and strict invalidation.

---

## 7. Linux driver reuse (driver-VM-only)

Linux kernel driver code is **GPLv2-only** and must **never** be copied into Bivdi's own (MPL-2.0) components. Reuse of Linux drivers happens **only** inside an isolated driver VM.

---

## 8. Open questions

- Driver availability is the existential risk: narrow hardware list vs. a Linux driver shim.
- GPU acceleration vs. a small TCB — there may be no way to have both.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
