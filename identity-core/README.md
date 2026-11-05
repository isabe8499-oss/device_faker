# identity-core

A small, host-testable Rust crate containing deterministic app-scoped identifier primitives.

## Current scope

- Validates a dotted package name and records the actual Android user ID as part of an `AppScope`.
- Derives a stable, synthetic app-instance identifier from a 32-byte caller-provided seed using HMAC-SHA-256.
- Formats the 128-bit result using UUID version 8 and the RFC variant bits.

The output is deterministic, not a random UUIDv4. It is an opaque synthetic value, **not** an implementation of GAID, Firebase Installations ID, Android ID, IMEI, IMSI, ICCID, a phone number, Widevine ID, or an attestation identifier.

## Integration boundary

This crate currently has no Zygisk, JNI, Android framework, system-property, CLI, or WebUI integration. Calling it does not change any OS or app-visible identifier. A later integration must explicitly opt in for a configured app and must obtain entropy for the seed from an appropriate platform CSPRNG outside this crate.

Pass the base package name and the actual Android user ID separately. For example, use `AppScope::new("com.example.app", 999)`, not `AppScope::new("com.example.app@999", 999)`.

Do not write seed values or derived identifiers to normal logs.
