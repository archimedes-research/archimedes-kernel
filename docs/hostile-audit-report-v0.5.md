# Hostile Audit Report for Minimum Kernel Footing v0.5

## Status

This report documents the results of a hostile review pass conducted by the implementing partner acting in an adversarial-review capacity.

This report **does not** constitute certification.  
Certification requires an independent authority or a formal acceptance step as defined in the Certification Authority Model.

---

## 1. Audit Scope

The audit covered:

- Source code of the `archimedes-kernel` crate.
- Public API and encapsulation boundaries.
- Movement protocol enforcement.
- Drift detection from birth state.
- Movement memory hash-chain integrity.
- MovementComposition view and verification.
- Sequence planning (`plan_sequence`, `PlannedSequence`).
- Reality fingerprint (`RealityFingerprint`, `fingerprint()`).
- Unified integrity report (`IntegrityReport`, `integrity_report()`).
- Event preflight validation (`would_accept()`).
- Test suite and failure behavior.
- Example binary behavior.

The audit boundary matches the certification boundary defined in `docs/hostile-audit-packet-v0.md` (updated for v0.5).

---

## 2. Audit Procedure

1. Ran `cargo test` and confirmed all 34 tests pass.
2. Ran `cargo run --example demo` and confirmed expected output.
3. Inspected `src/lib.rs` for `#![forbid(unsafe_code)]`.
4. Inspected `src/primitives/mod.rs` and confirmed `Reality` fields are `pub(crate)`, with only constructor and getters public.
5. Inspected `src/movement/mod.rs` and confirmed hash chain and `MovementComposition` are implemented.
6. Inspected `src/verification/mod.rs` and confirmed `drift_check()` uses `birth_boundary` and `birth_law`.
7. Inspected `src/verification/mod.rs` and confirmed `plan_sequence` operates on a clone and returns a `PlannedSequence`.
8. Inspected `src/verification/mod.rs` and confirmed `fingerprint()` uses standard-library hashing and includes identity, birth boundary, birth law, current state, initial state, and memory chain head.
9. Inspected `src/verification/mod.rs` and confirmed `integrity_report()` bundles fingerprint, drift, memory integrity, replay, continuity, state, and transition count.
10. Inspected `src/verification/mod.rs` and confirmed `would_accept()` correctly checks empty event, boundary inclusion, and law check without mutation.
11. Attempted to identify hidden mutation paths accessible from outside the crate.
12. Attempted to identify ways to tamper with `MovementMemory` without detection.
13. Reviewed the hash function for suitability.

---

## 3. Findings

### 3.1 No unsafe code

`#![forbid(unsafe_code)]` is present. No `unsafe` code exists.

**Result: Pass**

### 3.2 Encapsulation prevents external mutation

`Reality` fields are `pub(crate)`. External mutation is impossible through the public API.

**Result: Pass**

### 3.3 Movement order enforced

`perform_movement` follows the required order.

**Result: Pass**

### 3.4 Lawful failure behavior

All required failure conditions are covered by tests.

**Result: Pass**

### 3.5 Drift detection from birth state

`drift_check()` uses `birth_boundary` and `birth_law`. Hidden mutation between movements is detected.

**Result: Pass**

### 3.6 Movement memory integrity

Hash chain is implemented. Tampering with a transition is detected.

**Limitation:** `DefaultHasher` is not cryptographically secure. Acceptable for current threat model.

**Result: Pass with limitation**

### 3.7 MovementComposition

`MovementComposition` is implemented and verified against the memory chain. Tests cover valid range, invalid range, and tamper detection.

**Result: Pass**

### 3.8 Sequence planning

`plan_sequence` correctly clones the `Reality`, runs the sequence on the clone, and returns a `PlannedSequence`. The original is not mutated. Tests cover success, failure, and match with executed sequence.

**Result: Pass**

### 3.9 Reality fingerprint

`fingerprint()` returns a `RealityFingerprint` based on a structural hash. Tests confirm identical realities have identical fingerprints and mutation changes the fingerprint.

**Limitation:** `DefaultHasher` is not collision-resistant, so fingerprints are suitable for structural comparison but not for adversarial proof.

**Result: Pass with limitation**

### 3.10 Unified integrity report

`integrity_report()` successfully bundles fingerprint, drift check, memory integrity, replay, continuity, state, and transition count. Tests cover clean state and drift detection.

**Result: Pass**

### 3.11 Event preflight validation

`would_accept()` correctly returns:
- `true` for a lawful event from the current state.
- `false` for an unlawful event.
- `false` for an empty event.

The example output shows `would_accept = false` for the second event when checked before the first event has executed. This is expected, because `"done"` is only lawful after `"after"`. No issue found.

**Result: Pass**

### 3.12 ProofResult runtime checks

Some `ProofResult` fields are compile-time guarantees due to API design. Acceptable.

**Result: Pass with note**

---

## 4. Blocker Assessment

No blockers found.

---

## 5. Certification Readiness

The kernel is ready for external certification as **Minimum Kernel Footing v0.5**.

---

## 6. Reviewer Statement

- Review performed by: Implementing partner (adversarial pass)
- Date: 2026-08-20
- Result: No blockers found; limitations recorded.
- Signed: ____________________________
