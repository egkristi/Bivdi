# Bivdi — Networking

**Status:** Draft, grounded in the decided model ([`docs/decisions.md`](decisions.md)). This document elaborates the decided networking model. It asserts no not-yet-decided specifics.

---

## 1. Decided principles

1. **Identity-based networking.** Applications request a service by *identity and capability*, not IP address and port. The OS resolves identity to location and transport.
2. **Encryption by default.** Mutual authentication; encryption cannot be disabled for external connections.
3. **No raw sockets as ambient authority.** A component receives *flow capabilities* ("you may connect to this endpoint with this trust anchor"), never an unrestricted socket.
4. **Uniform, but not hidden.** The same API and capability model apply locally and remotely — but latency, timeouts, and partial failure are explicit in the types. The network is never disguised as local (`D-008`).

---

## 2. Flow capabilities, not sockets

There is no ambient network namespace in which to `bind(0.0.0.0)`. Listening requires a *listener capability* naming the specific interface and port.

A component with no flow capability has **no network access** — this is not a firewall rule that can be misconfigured; it is the absence of a name.

---

## 3. DNS is a capability decision

Name resolution and connection authority are one decision. DNS is a resolver service that hands back verified flow capabilities rather than raw addresses, closing the DNS-rebinding class of attacks.

---

## 4. Transport

The network stack is a userspace component, restartable, and not in the trusted computing base. It supports IPv6-first (IPv4 as compatibility), TCP, UDP, and QUIC as a first-class transport. *(Specific stack architecture beyond these decided properties is not yet specified.)*

Key material lives in a separate component that performs operations without releasing private keys, so a compromised network stack cannot exfiltrate them.

---

## 5. Distribution

- **Offline-first.** Devices are replicas and caches; disconnected operation is normal. CRDTs are used where meaningful, with explicit conflict resolution where they are not.
- **Clusters as explicit units.** Multiple Bivdi machines can form an explicitly configured cluster for workload and object placement — never an illusion of one giant machine.

---

## 6. Open questions (from [`SPEC.md`](../SPEC.md) §15)

- **Semantic conflicts in distributed data** — which object types use CRDTs vs. explicit conflict resolution.
- **Revocation** across the network — revocation of authority already handed out.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
