# Archimedes Kernel Public API

This document describes the public API of `archimedes-kernel` v2.

## 1. Scope

`archimedes-kernel` is a narrow Rust library for constructing and verifying deterministic state-transition systems.

It provides:

- explicit identity, boundary, law, and state;
- lawful movement;
- SHA-256 hash-chained movement memory;
- replay and continuity verification;
- drift detection;
- snapshots, diffs, planning, preflight, and simulation;
- versioned binary persistence;
- optional Ed25519-authenticated persistence.

It does not provide networking, database storage, UI, external identity verification, or production key management.

## 2. Dependency

After v2.0.0 is published to crates.io:

```toml
[dependencies]
archimedes-kernel = "2.0.0"
```

Local checkout:

```toml
[dependencies]
archimedes-kernel = { path = "." }
```

Applications using the signing examples directly also need compatible ed25519-dalek and rand dependencies.

## 3. Creating a Reality

```rust
use archimedes_kernel::primitives::{
    Boundary, Identity, Law, Reality, State,
};

let reality = Reality::new(
    Identity("example-reality".to_string()),
    Boundary {
        allowed_values: vec![
            "idle".to_string(),
            "ready".to_string(),
            "done".to_string(),
        ],
    },
    Law {
        allowed_transitions: vec![
            ("idle".to_string(), "ready".to_string()),
            ("ready".to_string(), "done".to_string()),
        ],
    },
    State {
        field: "idle".to_string(),
    },
);
```

The identity, boundary, law, state, birth configuration, and movement memory are held internally by Reality.

## 4. Performing movement

```rust
use archimedes_kernel::{
    movement::Event,
    perform_movement,
};

let event = Event {
    proposed_field: "ready".to_string(),
};

let proof = perform_movement(&mut reality, event)?;

assert!(proof.proof_status);
```

An event does not mutate state directly.

The kernel first evaluates the requested transition against the active Law. A rejected movement returns MovementError.

## 5. Sequential movement

```rust
use archimedes_kernel::{
    movement::Event,
    perform_movement_sequence,
};

let events = vec![
    Event {
        proposed_field: "ready".to_string(),
    },
    Event {
        proposed_field: "done".to_string(),
    },
];

let proofs =
    perform_movement_sequence(&mut reality, events)?;
```

Execution stops at the first rejected movement.

## 6. Read-only verification

```rust
let report = reality.verify();

assert!(report.replay.passed);
assert!(report.continuity.preserved);
assert!(report.memory_integrity);

let drift = reality.drift_check();
assert!(!drift.hidden_drift_required);

let fingerprint = reality.fingerprint();

let snapshot = reality.snapshot();
assert!(snapshot.matches_current(&reality));
```

memory_integrity() verifies the SHA-256 movement hash chain.

replay() reconstructs state from movement history.

continuity() reports whether replay reaches the current state.

drift_check() separately detects changes to active boundary and law and replay-visible state drift.

A RealityFingerprint is not a replacement for drift_check(). The fingerprint and drift report represent different integrity observations.

## 7. Planning and simulation

A sequence can be checked without mutating the original reality:

```rust
let report = reality.preflight_sequence(&events);
```

A full planned sequence can be produced with:

```rust
let planned = archimedes_kernel::plan_sequence(
    &reality,
    events.clone(),
)?;
```

Simulation produces a planned result, integrity report, optional movement composition, and fingerprint:

```rust
let simulation =
    archimedes_kernel::simulate_sequence(&reality, events)?;
```

## 8. Unsigned persistence

Version 2 uses Postcard.

```rust
use std::path::Path;
use archimedes_kernel::{
    load_reality,
    save_reality,
};

let path = Path::new("reality.bin");

save_reality(&reality, path)?;

let loaded = load_reality(path)?;

assert_eq!(reality, loaded);
```

Unsigned load_reality() checks movement-memory integrity and drift after decoding.

Unsigned persistence does not provide authenticity against an adversary capable of creating a new internally consistent artifact.

Snapshots can also be stored and loaded with:

```text
save_snapshot
load_snapshot
```

## 9. Signed persistence

Signed persistence uses Ed25519.

A v2 signed reality binds:

```text
ARCHIMEDES-KERNEL-SIGNED-REALITY-V2
+ protocol version
+ Reality
+ embedded authority public key
```

A signed snapshot uses a separate domain:

```text
ARCHIMEDES-KERNEL-SIGNED-SNAPSHOT-V2
```

Example:

```rust
use std::path::Path;

use archimedes_kernel::{
    load_signed_reality,
    save_signed_reality,
    sign_reality,
};

use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

let mut csprng = OsRng;
let signing_key = SigningKey::generate(&mut csprng);

let expected_public_key =
    signing_key.verifying_key().to_bytes();

let signed =
    sign_reality(&reality, &signing_key)?;

save_signed_reality(
    &signed,
    Path::new("signed-reality.bin"),
)?;

let loaded = load_signed_reality(
    Path::new("signed-reality.bin"),
    &expected_public_key,
)?;

assert_eq!(loaded.reality, reality);
assert_eq!(
    loaded.public_key,
    expected_public_key
);
```

use std::path::Path;

use archimedes_kernel::{
    load_signed_reality,
    save_signed_reality,
    sign_reality,
};

use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

let mut csprng = OsRng;
let signing_key = SigningKey::generate(&mut csprng);

let expected_public_key =
    signing_key.verifying_key().to_bytes();

let signed =
    sign_reality(&reality, &signing_key)?;

save_signed_reality(
    &signed,
    Path::new("signed-reality.bin"),
)?;

let loaded = load_signed_reality(
    Path::new("signed-reality.bin"),
    &expected_public_key,
)?;

assert_eq!(loaded.reality, reality);
assert_eq!(
    loaded.public_key,
    expected_public_key
);

Verification rejects:

- unsupported protocol versions
- wrong trusted public keys
- modified embedded public keys
- modified signed payloads
- invalid signatures
- invalid movement memory
- hidden drift

The expected public key must come from a trusted channel.

## 10. Persistence compatibility

Version 2 is not byte-compatible with the v1 bincode persistence format.

The crate does not automatically migrate v1 files.

The public constant:

```text
PERSISTENCE_VERSION
```

currently has the value:

```text
2
```

## 11. Security boundary

The kernel provides integrity mechanisms. It does not establish that caller-supplied identities, boundaries, or laws are correct in the external world.

A valid signature means the serialized v2 payload verifies against the supplied trusted Ed25519 public key.

It does not establish:

- signer intent
- signing-key custody
- external identity authenticity
- host integrity
- freshness
- revocation status
- rollback resistance

See `THREAT_MODEL.md`.

## 12. Unsafe code

The crate root declares:

```rust
#![forbid(unsafe_code)]
```

## 13. Error model

Movement failures return MovementError.

Persistence failures return PersistenceError.

Persistence errors include:

- IO failures
- serialization failures
- integrity failures
- signature failures
- unsupported persistence versions
- embedded/trusted public-key mismatch

## 14. Current limitations

The crate does not currently provide:

- multi-reality orchestration
- networking
- database integration
- UI
- key rotation
- key revocation
- replay prevention
- rollback prevention
- cross-version persistence migration

Use it as infrastructure, not as an end-user security product.
