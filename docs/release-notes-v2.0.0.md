# archimedes-kernel v2.0.0

## Summary

Version 2 hardens the kernel persistence and signing boundary.

The release replaces the unmaintained `bincode` dependency, introduces explicit persistence protocol versioning, separates signed artifact domains, and cryptographically binds the embedded authority public key into signed persistence records.

## Breaking changes

### Persistence encoding

v1:

```text
bincode

v2:

Postcard 1.1.3

Version 2 persistence files are not byte-compatible with v1 files.

No automatic v1 decoder or migration layer is included.

### Signed persistence envelope

The v1 implementation signed the serialized Reality or RealitySnapshot payload.

The embedded public_key field was not part of those signed bytes.

Version 2 signs an explicit envelope containing:

domain separator
protocol version
payload
authority public key

The embedded authority public key is therefore cryptographically bound to the signature.

## Domain separation

Signed realities use:

ARCHIMEDES-KERNEL-SIGNED-REALITY-V2

Signed snapshots use:

ARCHIMEDES-KERNEL-SIGNED-SNAPSHOT-V2

### Explicit protocol version

The public constant:

PERSISTENCE_VERSION

has value:

2

Unsigned persisted realities and snapshots carry a persistence version.

Signed realities and snapshots carry a protocol version.

Unsupported versions are rejected.

### Public-key consistency

Signed loading now requires:

embedded public key == externally trusted expected public key

A mismatch is rejected before the artifact is accepted.

## Dependency audit

bincode 1.3.3 was removed after RustSec advisory:

RUSTSEC-2025-0141
Bincode is unmaintained

The v2 persistence implementation uses:

postcard 1.1.3
default-features = false
features = ["alloc"]

The final v2 dependency audit reported no advisory.

## Verification evidence

The v2 hardening gate passed:

cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
cargo audit
cargo run --example demo

Test result:

61 passed
0 failed

New persistence tests include rejection of:

tampered signed reality payload
tampered signed snapshot payload
tampered embedded authority key
wrong expected authority key
unsupported signed protocol version
unsupported unsigned persistence version

Round-trip tests cover unsigned and signed realities and snapshots.

## Unchanged core

The movement hash chain and reality fingerprint do not depend on the persistence serializer.

The core SHA-256 movement-memory design remains separate from the v2 persistence encoding.

## Security boundary

Version 2 does not add:

key rotation
key revocation
multi-signature support
freshness proofs
replay prevention
rollback prevention
confidentiality
external identity authentication
host attestation

See THREAT_MODEL.md.

## Migration

Applications requiring access to historical v1 persistence artifacts should perform any required application-level extraction or conversion using v1 tooling before replacing it with v2.

ARCHIMEDES does not claim automatic wire compatibility between v1 and v2.
