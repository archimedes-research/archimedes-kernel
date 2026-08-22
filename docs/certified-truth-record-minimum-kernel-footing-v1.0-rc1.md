# Certified Truth Record: Minimum Kernel Footing v1.0-rc1

## Certification Boundary

The following are certified as **Minimum Kernel Footing v1.0-rc1**:

- Repository: `archimedes-kernel`
- Crate: `archimedes-kernel` (library)
- Dependencies: `sha2`, `serde`, `bincode`, `ed25519-dalek`, `rand`
- Modules:
  - `src/primitives/`
  - `src/movement/`
  - `src/verification/`
  - `src/persistence/`
- Example binary: `examples/demo.rs`
- Control documents:
  - `docs/repository-boundary-classification-v0.md`
  - `docs/review-ready-repository-structure-v0.md`
  - `docs/hostile-audit-packet-v0.md`
  - `docs/hostile-audit-report-v0.12.md`
  - `docs/public-api.md`
  - `docs/release-notes-v1.0-rc1.md`
  - This certification record

No other artifacts are inside this certification boundary.

---

## Evidence

- **Tests**: 55 unit tests pass (`cargo test`)
- **Formatting**: `cargo fmt --check` passes
- **Linting**: `cargo clippy -- -D warnings` passes
- **Example**: `cargo run --example demo` runs successfully
- **Hostile audit report**: `docs/hostile-audit-report-v0.12.md` found no blockers

---

## Limitations

1. Signed persistence does not yet include key rotation, revocation, or multi-signature schemes.
2. Persistence is binary; no schema evolution or migration yet.
3. This certification applies only to the minimum kernel footing v1.0-rc1.

---

## Certification Authority

- **Name**: Project Owner (Clement)
- **Acceptance date**: 2026-08-22

---

## Declaration

The kernel is certified as **Minimum Kernel Footing v1.0-rc1** within the above boundary only.

Any change outside this boundary invalidates this certification until re-certified.

---

## Acceptance Block

- [x] I have reviewed the evidence and audit report.
- [x] I accept the kernel as certified truth within the stated boundary.

**Signature**: ____________________  
**Date**: 2026-08-22
