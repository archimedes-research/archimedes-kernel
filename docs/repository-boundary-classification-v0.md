# Repository Boundary Classification Object v0

## Status

Accepted for Minimum Kernel Footing v0.  
Updated after cryptographic hashing (SHA‑256), unsigned persistence, and authenticated persistence using Ed25519 signatures.

## Classification

- Repository: single repository `archimedes-kernel`
- Workspace: one Rust workspace (current root)
- Crates: one library crate `archimedes-kernel`
- Dependencies: `sha2`, `serde`, `bincode`, `ed25519-dalek`, `rand`
- Module grouping:
  - `src/primitives/` — Reality, Identity, Boundary, Law, State
  - `src/movement/` — Event, LawCheck, Transition, MovementMemory, HashValue, hash_transition, MovementComposition
  - `src/verification/` — Inspection, Replay, Continuity, DriftCheck, ProofResult, MovementError, VerificationReport, detect_drift, PlannedSequence, plan_sequence, RealityFingerprint, fingerprint(), IntegrityReport, integrity_report(), would_accept(), SimulationReport, simulate_sequence, RealityDiff, diff(), PreflightReport, preflight_sequence, RealitySnapshot, snapshot(), RealitySnapshot::matches_current(), RealitySnapshot::diff_against()
  - `src/persistence/` — save_reality, load_reality, save_snapshot, load_snapshot, PersistenceError, SignedReality, SignedSnapshot, sign_reality, save_signed_reality, load_signed_reality, sign_snapshot, save_signed_snapshot, load_signed_snapshot
- Example binary: `examples/demo.rs` (demonstration only, not part of kernel crate)

## Boundary Rules

- No additional crates may be added without reclassification.
- No new modules outside the four groups without lawful opening.
- `src/primitives/` must not contain movement or verification logic.
- `src/movement/` must not contain verification logic.
- `src/verification/` may use primitives and movement types but must not mutate them.
- `src/persistence/` may serialize, deserialize, sign, and verify existing types but must not define new domain logic.
- External code may only construct `Reality` via `Reality::new(...)` and read state via getters. Direct mutation of `Reality` fields outside the crate is not possible because fields are `pub(crate)`.

## What is NOT included

- Production runtime
- Database
- Networking
- Concurrency
- Scheduler
- Lifecycle governance
- Migration machinery
- Multi-reality engine
