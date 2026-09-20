# Bivdi — Boot and Attestation

**Status:** Draft. The decided principle is **measured boot with storage sealed to the boot state**. Specific stage/PCR layouts, the shim, and remote-attestation encodings are **proposed** and marked open.

---

## 1. Decided principle

The boot chain is measured, and **storage is sealed to the boot measurement** ([`SPEC.md`](../SPEC.md) §13). A tampered system fails to unseal, so data stays encrypted.

---

## 2. The chain (conceptual)

```text
Platform firmware → boot loader → kernel → root supervisor → composition → steady state
```

Each stage measures the next. The storage key unseals only against the expected measurement set. *(The specific PCR/measured-boot-log layout is proposed, not decided.)*

---

## 3. Updates vs. tampering

Legitimate updates **re-seal** to the new expected measurement as part of the update transaction. This is what makes an update and a tamper distinguishable: an update changes the expected measurement through a controlled, authenticated path; a tamper does not.

---

## 4. Remote attestation

A verifier can request a signed quote proving *exactly which components are running, by content hash* — stronger than attesting a monolithic kernel image, because it covers the full authority composition.

*(The exact quote format and transport are **open**.)*

---

## 5. Root of trust

The root of trust is the platform firmware (UEFI Secure Boot / ARM TF-A / OpenSBI), with the root capability held in hardware and used only for recovery, key rotation, and ownership change.

---

## 6. Open questions

- Stage/PCR layout and shim signing (Secure Boot third-party CA vs. user-enrolled keys).
- Remote-attestation quote format and transport.

---

*Bivdi — nothing has ambient authority. Everything must ask.*
