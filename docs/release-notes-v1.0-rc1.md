# Release Notes: Archimedes Kernel v1.0-rc1

## Status

This is a release candidate for the minimum kernel.

## Highlights

- Lawful movement substrate
- SHA‑256 movement memory and fingerprints
- Persistent drift detection from birth state
- Read-only verification, snapshot, diff, preflight, and simulation
- Deterministic binary persistence
- Ed25519 authenticated persistence
- `#![forbid(unsafe_code)]`
- 55 passing tests
- No warnings under `cargo fmt --check` and `cargo clippy -- -D warnings`

## Changes since v0.12

No functional changes. This release candidate is the result of release hardening:

- Format verification added.
- Lint verification added.
- Public API documentation added.
- External user quickstart added.
- Release readiness confirmed.

## Known limitations

- No multi-reality engine.
- No database or networking.
- No key rotation or revocation.
- No schema migration for persisted data.

## Target audience

Rust developers building small, auditable, lawful-state systems such as:

- deterministic simulations;
- event-sourced kernels;
- auditable agents;
- rule-based digital worlds.
