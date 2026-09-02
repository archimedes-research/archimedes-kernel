# archimedes-kernel Public API — 3.0.0 Qualification Candidate

## Status

This document describes the ARCH-005 Engineering qualification candidate.

The crate version is `3.0.0`.

This candidate has not yet received independent Assurance and is not a qualification decision by itself.

The persistence protocol remains version `2` for valid persisted objects. Crate semantic version and persistence protocol version are intentionally separate identities.

## Reality construction

`Reality::new` is fallible.

Safe construction returns `Result<Reality, MovementError>` and rejects semantically invalid initial configurations.

Callers must explicitly handle construction failure.

`Reality::validate` exposes the central semantic validity contract.

Public deserialization of `Reality` is also validity-enforcing.

## Movement semantics

A successful movement must remain inside the active Boundary.

Movement failure is caller-visible failure-atomic: an `Err` does not commit the candidate state or candidate movement-memory transition.

Preflight evaluates the same movement semantics as execution on an isolated clone.

Cryptographic movement-memory integrity and semantic state continuity are distinct properties.

`MovementMemory::verify_integrity` checks cryptographic transition/hash linkage.

`MovementMemory::verify_semantic_continuity` checks state-machine continuity.

Replay requires initial-state origin, adjacent state continuity, law grounding, and final-state agreement.

## Snapshot semantics

`RealitySnapshot::validate` checks the self-consistency evidence available within the compact snapshot.

Snapshot validity does not claim independent reconstruction of MovementMemory that the snapshot does not contain.

## Persistence

Persistence uses Postcard protocol version `2`.

Loaders require complete-artifact framing and reject unexplained trailing bytes.

Safe Reality persistence enforces semantic validity.

Safe Snapshot persistence enforces snapshot self-consistency.

Local replacement is crash-conscious at the tested filesystem boundary:

1. serialize before destination mutation;
2. create a unique sibling temporary file;
3. write complete bytes;
4. sync the temporary file;
5. rename over the destination; and
6. sync the parent directory on Unix.

A post-rename directory-sync failure is represented as durability indeterminate because replacement may already have occurred.

This is not a distributed transaction or rollback/freshness protocol.

## Signed persistence

Ed25519 remains the signing primitive.

The expected public key is supplied externally by the caller.

For signed Reality loading, the kernel:

1. decodes a neutral wire representation;
2. checks protocol version;
3. checks the embedded key against the expected authority key;
4. verifies the signature over the wire representation; and
5. converts the authenticated wire into a semantically validated Reality.

An invalid signature and an authentic-but-semantically-invalid Reality are therefore distinct failure classes.

A valid signature proves authenticity relative to the expected public key. It does not make invalid semantics valid.

## Compatibility

The move from infallible to fallible `Reality::new` is a Rust API breaking change and is the reason for the crate major-version transition from `2.0.0` to `3.0.0`.

The valid version-2 persistence representation remains intentionally retained.

Artifacts that relied on previously accepted semantic invalidity or unexplained trailing data are intentionally rejected.

## Non-claims

The kernel does not claim to provide:

- external identity establishment;
- signer trust establishment;
- host integrity;
- confidentiality;
- freshness or rollback prevention;
- distributed consensus;
- database transaction semantics;
- network transport security; or
- correctness of caller-supplied business meaning beyond the kernel invariants it can evaluate.

## Qualification boundary

This document records Engineering behavior.

Independent Assurance and final ARCHIMEDES name fitness remain separate later decisions.
