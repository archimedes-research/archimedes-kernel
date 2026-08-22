# Archimedes Kernel Public API

This document gives external developers a compact entry point for using `archimedes-kernel`.

## 1. What this crate provides

`archimedes-kernel` is a minimal Rust library for constructing lawful, replayable, drift-resistant digital realities.

It enforces:

- explicit identity, boundary, law, and state;
- lawful movement only;
- cryptographic movement memory;
- drift detection from birth state;
- read-only verification, snapshot, and diff;
- deterministic persistence;
- optional authenticated persistence using Ed25519.

The kernel is intentionally narrow. It does not include database, networking, UI, or domain-specific behavior.

## 2. Adding the dependency

In `Cargo.toml`:

```toml
[dependencies]
archimedes-kernel = { path = "." }

## 3. Creating a Reality

use archimedes_kernel::primitives::{Boundary, Identity, Law, Reality, State};

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
    State { field: "idle".to_string() },
);

## 4. Performing lawful movement

use archimedes_kernel::{movement::Event, perform_movement};

let mut reality = /* create as above */;
let event = Event { proposed_field: "ready".to_string() };
let proof = perform_movement(&mut reality, event)?;
assert!(proof.proof_status);

Unlawful events are rejected and return an error.

## 5. Sequential movement

use archimedes_kernel::{movement::Event, perform_movement_sequence};

let events = vec![
    Event { proposed_field: "ready".to_string() },
    Event { proposed_field: "done".to_string() },
];

let proofs = perform_movement_sequence(&mut reality, events)?;

## 6. Read-only verification

let report = reality.verify();
assert!(report.replay.passed);
assert!(report.continuity.preserved);
assert!(report.memory_integrity);

let drift = reality.drift_check();
assert!(!drift.hidden_drift_required);

let fingerprint = reality.fingerprint();

let snapshot = reality.snapshot();
assert!(snapshot.matches_current(&reality));

## 7. Persistence

Unsigned:

use std::path::Path;
use archimedes_kernel::{save_reality, load_reality};

let path = Path::new("reality.bin");
save_reality(&reality, path)?;
let loaded = load_reality(path)?;
assert_eq!(reality, loaded);

Signed:

use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use archimedes_kernel::{sign_reality, save_signed_reality, load_signed_reality};

let mut csprng = OsRng;
let signing_key = SigningKey::generate(&mut csprng);
let public_key = signing_key.verifying_key().to_bytes();

let signed = sign_reality(&reality, &signing_key)?;
save_signed_reality(&signed, Path::new("signed-reality.bin"))?;

let loaded_signed = load_signed_reality(Path::new("signed-reality.bin"), &public_key)?;
assert_eq!(loaded_signed.reality, reality);

## 8. Important guarantees
#![forbid(unsafe_code)] is set in the crate.

All movement goes through the lawful movement functions.

Reality fields are private to the crate; external mutation is impossible.

Movement memory uses SHA‑256.

Signed persistence uses Ed25519.

## 9. Limitations
This crate is a minimum kernel, not a production application. It does not yet include:

multi-reality orchestration;

networking;

database;

UI;

key management or rotation;

schema migration.

Use it as a substrate, not as an end-user product.
