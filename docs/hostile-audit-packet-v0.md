# Hostile Audit Packet for Minimum Kernel Footing v0

## Status

This packet is **not** a certification.

It is a request for hostile review as required by the Certified Truth Rule:

- lawful opening;
- lawful embodiment;
- hostile review or equivalent lawful audit;
- lawful acceptance;
- exact certification boundary.

The kernel is currently in the **implemented** state and is now being submitted for **hostile review**.

---

## 1. Certification Boundary

The certification boundary is **exactly** the following:

- Repository: `archimedes-kernel`
- Crate: `archimedes-kernel` (library)
- Dependencies: `sha2`, `serde`, `bincode`, `ed25519-dalek`, `rand`
- Modules:
  - `src/primitives/` — `Identity`, `Boundary`, `Law`, `State`, `Reality`
  - `src/movement/` — `Event`, `LawCheck`, `Transition`, `MovementMemory`, `HashValue`, `hash_transition`, `MovementComposition`
  - `src/verification/` — all verification types and functions, including snapshot comparison
  - `src/persistence/` — unsigned and signed persistence functions and types
- Example binary: `examples/demo.rs` (demonstration only; not part of the library crate)
- Control documents:
  - `docs/repository-boundary-classification-v0.md`
  - `docs/review-ready-repository-structure-v0.md`
  - This audit packet

No other artifacts are inside the certification boundary.

---

## 2. Active Governing Truth

The following govern this audit:

- **Archimedes Operational Policy v1**
- **Archimedes Implementation Guideline**
- **Repository Boundary Classification Object v0**
- **Review-Ready Repository Structure Object v0**

---

## 3. Implemented Capabilities

The kernel currently implements and enforces:

1. A `Reality` with:
   - Stable `Identity`
   - `Boundary` (allowed state values)
   - `Law` (allowed transitions)
   - Current `State`
   - `initial_state`
   - `birth_boundary` and `birth_law` for persistent drift detection
   - `MovementMemory` with SHA‑256 hash-chained `Transition`s

2. Lawful movement functions:
   - `perform_movement(reality, event)` — single movement
   - `perform_movement_sequence(reality, events)` — multiple sequential movements

3. Read-only verification API:
   - `inspect()`, `replay()`, `continuity()`, `memory_integrity()`, `verify()`, `drift_check()`, `fingerprint()`, `integrity_report()`, `would_accept()`, `diff()`, `preflight_sequence()`, `snapshot()`

4. Movement composition:
   - `MovementMemory::compose(start, end)` — creates a `MovementComposition` view
   - `MovementComposition::verify(&memory)` — verifies the composition

5. Sequence planning and simulation:
   - `plan_sequence(&reality, events)` — dry‑run on a clone
   - `simulate_sequence(&reality, events)` — full simulation on a clone

6. Snapshot comparison:
   - `RealitySnapshot::matches_current(&reality)`
   - `RealitySnapshot::diff_against(&reality)`

7. Unsigned persistence:
   - `save_reality`, `load_reality`, `save_snapshot`, `load_snapshot` using `bincode`
   - Loading verifies movement memory integrity and drift check; rejects tampered files.

8. Authenticated persistence:
   - `SignedReality`, `SignedSnapshot` wrappers with Ed25519 public key and signature.
   - `sign_reality`, `sign_snapshot`, `save_signed_reality`, `load_signed_reality`, `save_signed_snapshot`, `load_signed_snapshot`.
   - Loading verifies signature against the expected public key, then integrity and drift.

9. Drift detection:
   - Hidden state mutation, boundary growth, law growth, permission drift

10. Crate-level `#![forbid(unsafe_code)]`

---

## 4. Required Pass Values and Evidence

The required pass values from v0.11 remain, plus:

| Capability | Test Name |
|---|---|
| Signed reality roundtrip works | `signed_reality_roundtrip_works` |
| Signed reality rejects wrong key | `signed_reality_rejects_wrong_key` |
| Signed snapshot roundtrip works | `signed_snapshot_roundtrip_works` |

---

## 5. Required Failure Conditions and Evidence

| Failure Condition | Test Name |
|---|---|
| Tampered persistence file rejected | `tampered_reality_file_is_rejected` |
| Signed reality wrong key rejected | `signed_reality_rejects_wrong_key` |

---

## 6. How to Run the Tests

cargo test

Expected: 55 tests pass.

To run the demonstration:

cargo run --example demo


Expected output includes signed persistence.

## 7. Reviewer Checklist
A hostile reviewer should verify:

□ Run cargo test and confirm all tests pass.
□ Inspect Cargo.toml and confirm only the five listed dependencies.
□ Inspect src/lib.rs and confirm #![forbid(unsafe_code)].
□ Inspect src/persistence/mod.rs and confirm signed loading verifies signature and integrity.
□ Run the example and confirm signed persistence works.


## 8. Acceptance Criteria
After hostile review, the kernel may be accepted as Minimum Kernel Footing v0.12 certified truth if all tests pass and no blockers are found.


## 9. Auditor Signature
Reviewer name/role: ____________________________

Date: ________________

Result (Accept/Reject): ________________

Notes: ________________
