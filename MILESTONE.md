# Milestone 1: Published Kernel and External Audit Tool

## Completed

- `archimedes-kernel` v1.0.0 published to crates.io
- GitHub repositories:
  - archimedes-kernel
  - archimedes-cli
  - archimedes-audit
- Cryptographic SHA-256 movement memory
- Ed25519 signed persistence
- External CLI proof of use
- External auditable workflow tool with signed output
- 55 kernel tests passing
- `cargo fmt --check` and `cargo clippy -- -D warnings` clean

## What this proves

The Archimedes property is not a private experiment. It is a usable, published substrate with a real external consumer.

## Next possible phase

- Build a larger simulation or workflow product on top.
- Add multi-reality coordination.
- Add external certification tooling.

---

# Milestone 2: Persistence and Signing Boundary Hardening

## Completed

- kernel source advanced to `archimedes-kernel` v2.0.0
- `bincode` removed
- Postcard introduced for versioned binary persistence
- explicit persistence/protocol version added
- signed reality and signed snapshot domains separated
- embedded authority public key cryptographically bound to signed payload
- expected authority public-key consistency enforced
- unsupported persistence versions rejected
- tampered signed payload tests added
- tampered embedded-key tests added
- 61 kernel tests passing
- release build clean
- RustSec dependency audit clean
- complete runtime example passing

## Compatibility boundary

The v2 persistence format is intentionally not byte-compatible with v1.

No automatic v1 persistence migration is provided.

## Security boundary

Version 2 strengthens persistence integrity and authenticity.

It does not add external identity authentication, key rotation, revocation, replay protection, rollback protection, host attestation, or confidentiality.

See:

- `THREAT_MODEL.md`
- `SECURITY.md`
- `docs/release-notes-v2.0.0.md`
