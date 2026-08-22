# Certified Truth Record: Minimum Kernel Footing v0.11

## Certification Boundary

The following are certified as **Minimum Kernel Footing v0.11**:

- Repository: `archimedes-kernel`
- Crate: `archimedes-kernel` (library)
- Dependencies: `sha2`, `serde`, `bincode`
- Modules:
  - `src/primitives/` — `Identity`, `Boundary`, `Law`, `State`, `Reality`
  - `src/movement/` — `Event`, `LawCheck`, `Transition`, `MovementMemory`, `HashValue`, `hash_transition`, `MovementComposition`
  - `src/verification/` — all verification types and functions
  - `src/persistence/` — save/load functions and `PersistenceError`
- Example binary: `examples/demo.rs`
- Control documents:
  - `docs/repository-boundary-classification-v0.md`
  - `docs/review-ready-repository-structure-v0.md`
  - `docs/hostile-audit-packet-v0.md`
  - `docs/hostile-audit-report-v0.11.md`
  - This certification record

No other artifacts are inside this certification boundary.

---

## Evidence

- **Tests**: 52 unit tests pass (`cargo test`)
- **Example**: `cargo run --example demo` works as expected, including persistence.
- **Hostile audit report**: `docs/hostile-audit-report-v0.11.md` found no blockers.

---

## Limitations

1. Persistence is binary and in-process; no cross-version migration or schema evolution yet.
2. Loading verifies integrity but does not cryptographically sign the file.
3. This certification applies only to the minimum kernel footing v0.11.

---

## Certification Authority

- **Name**: Project Owner (Clement)
- **Acceptance date**: 2026-08-21

---

## Declaration

The kernel is certified as **Minimum Kernel Footing v0.11** within the above boundary only.

Any change outside this boundary invalidates this certification until re-certified.

---

## Acceptance Block

- [x] I have reviewed the evidence and audit report.
- [x] I accept the kernel as certified truth within the stated boundary.

**Signature**: ____________________  
**Date**: 2026-08-21
