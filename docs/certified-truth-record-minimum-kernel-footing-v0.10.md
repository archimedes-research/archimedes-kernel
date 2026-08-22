# Certified Truth Record: Minimum Kernel Footing v0.10

## Certification Boundary

The following are certified as **Minimum Kernel Footing v0.10**:

- Repository: `archimedes-kernel`
- Crate: `archimedes-kernel` (library)
- Dependencies: `sha2`
- Modules:
  - `src/primitives/` — `Identity`, `Boundary`, `Law`, `State`, `Reality`
  - `src/movement/` — `Event`, `LawCheck`, `Transition`, `MovementMemory`, `HashValue`, `hash_transition`, `MovementComposition`
  - `src/verification/` — all verification types and functions including snapshot comparison
- Example binary: `examples/demo.rs`
- Control documents:
  - `docs/repository-boundary-classification-v0.md`
  - `docs/review-ready-repository-structure-v0.md`
  - `docs/hostile-audit-packet-v0.md`
  - `docs/hostile-audit-report-v0.10.md`
  - This certification record

No other artifacts are inside this certification boundary.

---

## Evidence

- **Tests**: 49 unit tests pass (`cargo test`)
- **Example**: `cargo run --example demo` works as expected.
- **Hostile audit report**: `docs/hostile-audit-report-v0.10.md` found no blockers.

---

## Limitations

1. SHA‑256 provides collision resistance, but the kernel remains in-process only; no persistence or networked verification yet.
2. Some `ProofResult` fields are compile-time guarantees.
3. This certification applies only to the minimum kernel footing v0.10.

---

## Certification Authority

- **Name**: Project Owner (Clement)
- **Acceptance date**: 2026-08-21

---

## Declaration

The kernel is certified as **Minimum Kernel Footing v0.10** within the above boundary only.

Any change outside this boundary invalidates this certification until re-certified.

---

## Acceptance Block

- [x] I have reviewed the evidence and audit report.
- [x] I accept the kernel as certified truth within the stated boundary.

**Signature**: ____________________  
**Date**: 2026-08-21
