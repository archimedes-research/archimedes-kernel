# Certified Truth Record: Minimum Kernel Footing v0

## Certification Boundary

The following are certified as **Minimum Kernel Footing v0**:

- Repository: `archimedes-kernel`
- Crate: `archimedes-kernel` (library)
- Modules:
  - `src/primitives/` — `Identity`, `Boundary`, `Law`, `State`, `Reality`
  - `src/movement/` — `Event`, `LawCheck`, `Transition`, `MovementMemory`, `HashValue`, `hash_transition`
  - `src/verification/` — `Inspection`, `Replay`, `Continuity`, `DriftCheck`, `ProofResult`, `MovementError`, `VerificationReport`, `detect_drift`
- Example binary: `examples/demo.rs` (demonstration only)
- Control documents:
  - `docs/repository-boundary-classification-v0.md`
  - `docs/review-ready-repository-structure-v0.md`
  - `docs/hostile-audit-packet-v0.md`
  - `docs/hostile-audit-report-v0.md`
  - This certification record

No other artifacts are inside this certification boundary.

---

## Evidence

- **Tests**: 21 unit tests pass (`cargo test`)
- **Example**: `cargo run --example demo` produces the expected output, including:
  - Movement sequence succeeded.
  - Verification report: replay passed, continuity preserved, memory integrity true.
  - Drift check: no hidden drift.
- **Hostile audit report**: `docs/hostile-audit-report-v0.md` found no blockers.

---

## Limitations

The following limitations are accepted as part of this certification:

1. The hash chain uses `DefaultHasher`, which is not cryptographically secure. It is suitable for detecting accidental or uncoordinated tampering but not malicious in‑process code with the ability to recompute hashes.
2. Some `ProofResult` fields (e.g., `reality_exists`, `event_directly_mutated_state`) are compile‑time guarantees enforced by the API design rather than runtime checks.
3. This certification applies only to the minimum kernel footing, not to any future expansion or production features.

---

## Certification Authority

- **Name**: Project Owner (Clement)
- **Acceptance date**: 2026‑08‑20

---

## Declaration

The kernel is certified as **Minimum Kernel Footing v0** within the above boundary only.

Any change outside this boundary invalidates this certification until re‑certified.

---

## Acceptance Block

- [x] I have reviewed the evidence and audit report.
- [x] I accept the kernel as certified truth within the stated boundary.

**Signature**: ____________________  
**Date**: 2026‑08‑20
