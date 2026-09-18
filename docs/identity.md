# Bivdi — Identity

**Status:** Draft, grounded in the decided model. Identity is a core design principle ("identity everywhere"). This document elaborates the decided identity service. Deep cryptographic machinery (e.g., zero-knowledge proof encodings) is **not yet specified**.

---

## 1. Decided principle

**Identity everywhere.** People, devices, workloads, services, objects, and organizations have identity. Identity is a *system service*, not something each application reinvents.

---

## 2. Identity kinds

| Kind | Example |
|---|---|
| **Person** | A human user |
| **Device** | A laptop, phone, server, node |
| **Workload** | A running application instance |
| **Service** | A long-lived system or application service |
| **Object** | A data object in the object store |
| **Organization** | A tenant or enterprise |

---

## 3. Cryptographic identity

Cryptographic identities underpin nodes and services where appropriate. Identity is the basis for:

- mutual authentication between components,
- authority delegation (capabilities are granted *to* identities),
- attribution in the provenance log.

Identity is a property of the channel, not a claim in a payload: a server identifies a client by an unforgeable badge set by the kernel, never by a self-reported username.

---

## 4. Petnames

Global, readable namespaces (DNS, usernames) require a central authority and invite phishing. Bivdi uses **petnames**: users assign *local* names to cryptographically identified parties.

- The system always displays the **user's own name** for a party, never the party's self-claimed name.
- Global names are used for *discovery*, never as a basis for *trust*.

---

## 5. Selective disclosure

A service can verify a property without receiving the underlying data. For example, a service can confirm "age ≥ 18" without receiving the birth date.

Applications receive only the identity attributes they need — never a wholesale dump of an identity.

---

## 6. Environments

An **environment** is an isolated context with its own applications, credentials, network access, storage, policy, and identity (e.g., personal, work, development, testing, travel). This is described further in the state/identity integration; environments are a first-class isolation boundary.

---

## 7. Open questions

- The concrete encoding of selective-disclosure proofs (not yet specified).
- How environments compose with the capability runtime at the detail level (part of spec v0.1).

---

*Bivdi — nothing has ambient authority. Everything must ask.*
