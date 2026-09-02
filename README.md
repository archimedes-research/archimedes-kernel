# Archimedes Kernel

## ARCH-005 qualification candidate

The current Engineering branch is an unreleased `archimedes-kernel` `3.0.0` qualification candidate.

`Reality::new` is now fallible and semantic validity is enforced at safe construction, public deserialization, unsigned persistence, signed persistence, and snapshot boundaries.

The crate major version is `3.0.0`; the valid persistence protocol remains version `2`.

Current qualified-behavior documentation is in `docs/public-api-v3.0.md`.

This Engineering state has not yet received independent Assurance and is not a final ARCHIMEDES name-fitness decision.


A minimal Rust kernel for deterministic state transition, tamper-evident movement memory, replay, continuity verification, drift detection, and versioned persistence.

## Status

Current source: **archimedes-kernel v2.0.0**

Research infrastructure. This is a substrate, not an end-user security product.

## What the kernel establishes

The kernel enforces the following movement order:

```text
Reality
→ Identity
→ Boundary
→ Law
→ State
→ Event
→ LawCheck
→ Transition
→ MovementMemory
→ Inspection
→ Replay
→ Continuity
→ DriftCheck
→ ProofResult
```

A movement succeeds only when the requested state transition is permitted by the active law.

The kernel also provides:

- SHA-256 hash-chained movement memory;
- replay-based continuity verification;
- detection of state, boundary, law, and permission drift;
- snapshots, fingerprints, diffs, preflight checks, and simulations;
- versioned binary persistence;
- optional Ed25519-authenticated persistence.

## What the kernel does not establish

The kernel does not prove that:

- an identity string corresponds to a real external actor;
- a boundary or law is semantically correct;
- the signer is trustworthy;
- a private signing key is uncompromised;
- a host running the kernel is uncompromised;
- a previously valid signed artifact is fresh rather than replayed or rolled back.

See `THREAT_MODEL.md` for the complete boundary.

## Quickstart

For the published v2.0.0 crate:

```toml
[dependencies]
archimedes-kernel = "2.0.0"
```

For a local source checkout:

```toml
[dependencies]
archimedes-kernel = { path = "." }
```

Create a reality and perform lawful movement:

```rust
use archimedes_kernel::{
    movement::Event,
    perform_movement_sequence,
    primitives::{Boundary, Identity, Law, Reality, State},
};

let mut reality = Reality::new(
    Identity("demo".to_string()),
    Boundary {
        allowed_values: vec![
            "before".to_string(),
            "after".to_string(),
            "done".to_string(),
        ],
    },
    Law {
        allowed_transitions: vec![
            ("before".to_string(), "after".to_string()),
            ("after".to_string(), "done".to_string()),
        ],
    },
    State {
        field: "before".to_string(),
    },
).expect("valid Reality construction");

let events = vec![
    Event {
        proposed_field: "after".to_string(),
    },
    Event {
        proposed_field: "done".to_string(),
    },
];

let proofs =
    perform_movement_sequence(&mut reality, events).unwrap();

assert!(proofs.iter().all(|proof| proof.proof_status));
assert_eq!(reality.state().field, "done");
```

Run the test suite:

```bash
cargo test
```

Run the complete example:

```bash
cargo run --example demo
```

## Verification

Read-only verification is available directly from a Reality:

```rust
let report = reality.verify();

assert!(report.memory_integrity);
assert!(report.replay.passed);
assert!(report.continuity.preserved);

let drift = reality.drift_check();
assert!(!drift.hidden_drift_required);
```

Movement memory is hash-chained with SHA-256.

Replay independently reconstructs state from recorded transitions and compares the result with current state.

Drift detection separately compares active boundary and law against their birth values.

## Persistence v2

Version 2 uses Postcard binary serialization.

Unsigned persistence provides serialization plus kernel consistency checks.

It does not provide cryptographic authenticity against an attacker capable of replacing a complete artifact and recomputing its internal data.

Signed persistence uses Ed25519 and binds:

```text
domain separator
+ protocol version
+ persisted payload
+ authority public key
```

Signed reality and signed snapshot records use separate domain separators.

Verification requires an externally trusted expected public key. The embedded public key must match that expected key.

## Compatibility

The v2 persistence format is intentionally incompatible with the v1 bincode format.

The kernel does not contain a v1 migration decoder.

Applications that need to preserve v1 persisted data should perform any required application-level migration using v1 tooling before adopting v2.

PERSISTENCE_VERSION is exported by the crate and is currently:

```text
2
```

## Module structure

```text
src/
├── primitives/    Reality, Identity, Boundary, Law, State
├── movement/      Event, LawCheck, Transition, MovementMemory,
│                  HashValue, MovementComposition
├── verification/  Inspection, Replay, Continuity, DriftCheck,
│                  ProofResult, VerificationReport, PlannedSequence,
│                  SimulationReport, RealityFingerprint,
│                  IntegrityReport, PreflightReport,
│                  RealitySnapshot, RealityDiff
└── persistence/   unsigned and authenticated versioned persistence
```

## Security properties

The crate contains:

```rust
#![forbid(unsafe_code)]
```

External callers are not given direct mutable access to the internal fields of Reality.

Cryptographic authenticity is provided only by the signed persistence API and only relative to a trusted expected Ed25519 public key.

See:

- `THREAT_MODEL.md`
- `SECURITY.md`
- `docs/public-api.md`
- `docs/release-notes-v2.0.0.md`

## Historical evidence

The repository preserves the audit and certification documents produced during development of the original minimum kernel.

These documents describe the implementation as it existed at those historical points and are intentionally not rewritten to describe v2.

Notable historical records include:

- `docs/certified-truth-record-minimum-kernel-footing-v1.0-rc1.md`
- `docs/hostile-audit-report-v0.12.md`

## Current limitations

No multi-reality orchestration.

No networking or database layer.

No production key-management system.

No key rotation or revocation.

No replay/rollback protection for previously valid signed persistence artifacts.

No automated migration from the v1 persistence format.

No claim that caller-supplied identities, laws, or boundaries are externally correct.

ARCHIMEDES

Truth Before Construction.
