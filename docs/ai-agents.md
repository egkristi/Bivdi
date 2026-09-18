# Bivdi — AI Agents

**Status:** Draft, grounded in the decided model (`README.md` §16). This document elaborates how AI agents run as first-class, constrained citizens. It asserts no not-yet-decided specifics.

---

## 1. Decided principle

**AI without unrestricted authority** (core invariant): agents are capability-constrained, time-limited, quota-bound, and fully provenance-logged. **"AI proposes, the OS enforces."**

An agent is a workload like any other — it holds only what it is handed, and it can never become root because root does not exist.

---

## 2. The threat: the confused deputy

An agent acting with a user's full authority is a textbook **confused deputy**: a prompt-injection hidden in a document can trick it into misusing access it was given for another purpose.

Bivdi's answer is structural, not advisory: an agent receives exactly the authority needed for its task, and **nothing more**.

---

## 3. The model

```text
User / organization
        │  delegates (attenuated, time-limited, quota-bound)
        ▼
     Agent host
        │  can only use and re-delegate what it holds
        ▼
 Tools, objects, services
        │
        ▼
 Provenance log (who, what, with which capability)
```

---

## 4. Decided constraints

- **Attenuated delegation.** An agent gets, for example, write access to one calendar entry and read access to one email thread — not the whole calendar and inbox.
- **Time-limited.** Capabilities expire when the task ends or a deadline passes.
- **Quota-bound.** Hard limits on CPU, memory, network calls, cost, and number of actions.
- **No escalation.** An agent can never delegate more than it holds.
- **Content is data, not instructions.** What an agent reads grants it no new rights; only the user or an authorized policy can widen access.
- **Irreversible actions require confirmation.** Sending, deleting, paying, and publishing can require explicit human approval, defined as policy.
- **Full provenance.** Every write is traceable to the agent, the task, and the capability chain.

---

## 5. AI proposes, the OS enforces

A natural-language request ("set up a PostgreSQL dev environment") becomes an explicit, reviewable **plan** that flows through the state engine and the capability runtime like any other change. The AI layer has no privileges of its own.

---

## 6. Semantic indexing without an all-seeing indexer

Semantic search over "everything a user has" conflicts with least privilege. Bivdi resolves this by scoping indexing per environment and per capability domain, with the indexer as a component holding only narrow read capabilities, and results filtered against the *requester's* capabilities — not the indexer's.

---

## 7. Open questions (from `README.md` §17)

- **Agent policy** — which actions always require human confirmation, and how it is expressed.
- **Semantic indexing** — how to offer useful cross-data search without a broad-access indexer (partially addressed above; not fully resolved).

---

*Bivdi — nothing has ambient authority. Everything must ask.*
