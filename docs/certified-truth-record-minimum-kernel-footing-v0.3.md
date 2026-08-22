# Certified Truth Record: Minimum Kernel Footing v0.3

## Certification Boundary

The following are certified as **Minimum Kernel Footing v0.3**:

- Repository: `archimedes-kernel`
- Crate: `archimedes-kernel` (library)
- Modules:
  - `src/primitives/` — `Identity`, `Boundary`, `Law`, `State`, `Reality`
  - `src/movement/` — `Event`, `LawCheck`, `Transition`, `MovementMemory`, `HashValue`, `hash_transition`, `MovementComposition`
  - `src/verification/` — `Inspection`, `Replay`, `Continuity`, `DriftCheck`, `ProofResult`, `MovementError`, `VerificationReport`, `detect_drift`, `PlannedSequence`, `plan_sequence`, `RealityFingerprint`, `fingerprint()`
- Example binary: `examples/demo.rs` (demonstration only)
- Control documents:
  - `docs/repository-boundary-classification-v0.md`
  - `docs/review-ready-repository-structure-v0.md`
  - `docs/hostile-audit-packet-v0.md`
  - `docs/hostile-audit-report-v0.3.md`
  - This certification record

No other artifacts are inside this certification boundary.

---

## Evidence

- **Tests**: 29 unit tests pass (`cargo test`)
- **Example**: `cargo run --example demo` produces the expected output, including:
  - Planned Sequence section.
  - Movement sequence succeeded.
  - Verification report: replay passed, continuity preserved, memory integrity true.
  - Drift check: no hidden drift.
  - Movement composition verified.
  - Reality fingerprint displayed.
- **Hostile audit report**: `docs/hostile-audit-report-v0.3.md` found no blockers.

---

## Limitations

1. The hash chain and fingerprint use `DefaultHasher`, which is not cryptographically secure.
2. Some `ProofResult` fields are compile-time guarantees enforced by API design.
3. This certification applies only to the minimum kernel footing v0.3, not to future expansion.

---

## Certification Authority

- **Name**: Project Owner (Clement)
- **Acceptance date**: 2026-08-20

---

## Declaration

The kernel is certified as **Minimum Kernel Footing v0.3** within the above boundary only.

Any change outside this boundary invalidates this certification until re-certified.

---

## Acceptance Block

- [x] I have reviewed the evidence and audit report.
- [x] I accept the kernel as certified truth within the stated boundary.

**Signature**: ____________________  
**Date**: 2026-08-20
