# Archimedes Kernel

A minimal Rust kernel for disciplined digital reality construction.

## Status

Minimum Kernel Footing v1.0-rc1 — release candidate.

## What this kernel proves

The kernel implements and enforces the following movement order:

Reality → Identity → Boundary → Law → State
→ Event → LawCheck → Transition → MovementMemory
→ Inspection → Replay → Continuity → DriftCheck
→ ProofResult

It refuses:

- unlawful transitions
- missing identity, boundary, law, or state
- state outside the allowed boundary
- transitions not grounded in a lawful check
- hidden drift
- tampered movement memory

## Quickstart

Add the dependency to `Cargo.toml`:

```toml
[dependencies]
archimedes-kernel = { path = "." }

Create and move a reality:

use archimedes_kernel::{
    movement::Event,
    perform_movement_sequence,
    primitives::{Boundary, Identity, Law, Reality, State},
};

let mut reality = Reality::new(
    Identity("demo".to_string()),
    Boundary {
        allowed_values: vec!["before".to_string(), "after".to_string(), "done".to_string()],
    },
    Law {
        allowed_transitions: vec![
            ("before".to_string(), "after".to_string()),
            ("after".to_string(), "done".to_string()),
        ],
    },
    State { field: "before".to_string() },
);

let events = vec![
    Event { proposed_field: "after".to_string() },
    Event { proposed_field: "done".to_string() },
];

let proofs = perform_movement_sequence(&mut reality, events).unwrap();
assert!(proofs.iter().all(|p| p.proof_status));

Run the tests:

cargo test

Run the example:

cargo run --example demo

For more, see docs/public-api.md.

Module structure

src/
├── primitives/    Reality, Identity, Boundary, Law, State
├── movement/      Event, LawCheck, Transition, MovementMemory, HashValue, MovementComposition
├── verification/  Inspection, Replay, Continuity, DriftCheck, ProofResult, VerificationReport,
│                  PlannedSequence, SimulationReport, RealityFingerprint, IntegrityReport,
│                  PreflightReport, RealitySnapshot, RealityDiff
└── persistence/   save_reality, load_reality, save_snapshot, load_snapshot,
                   sign_reality, sign_snapshot, signed persistence

Certified Truth
Minimum Kernel Footing v1.0-rc1 is certified as described in:

docs/certified-truth-record-minimum-kernel-footing-v1.0-rc1.md

The certification boundary, evidence, and limitations are documented there.

Hostile audit
A hostile audit report is available:

docs/hostile-audit-report-v0.12.md

Known limitations
No multi-reality engine.

No database or networking.

No key rotation or revocation.

No schema migration for persisted data.

This is a substrate, not an end-user product.
