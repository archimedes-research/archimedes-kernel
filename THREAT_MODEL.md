# Threat Model

## Scope

This document defines the security boundary of `archimedes-kernel` v2.

The kernel is a deterministic state-transition and integrity substrate.

It is not a complete security system.

## Protected properties

### Lawful movement

State movement through the public movement API is accepted only when the active law permits the requested transition.

### Movement-memory integrity

Recorded transitions form a SHA-256 hash chain.

Modification of recorded transition data without correctly rebuilding the chain is detected by `memory_integrity()`.

### Replay continuity

Recorded movement can be replayed from initial state.

A mismatch between replayed state and current state causes continuity verification to fail.

### Configuration drift

The kernel retains birth boundary and birth law.

`drift_check()` compares those values with the active boundary and law and reports hidden boundary growth, law growth, permission drift, and replay-visible state mutation.

### Signed persistence

Signed v2 realities and snapshots use Ed25519.

The signature binds:

```text
domain separator
protocol version
payload
authority public key
```

Signed realities and signed snapshots use separate domain separators.

Verification requires an externally supplied expected public key and rejects an embedded public key that does not match it.

## Trust assumptions

The verifier must obtain the expected authority public key through a trusted channel.

The authority private key is assumed to remain secret.

The process performing signing is assumed not to be malicious or compromised.

The host executing the kernel is assumed to execute the intended code correctly.

Cryptographic primitives and their implementations are assumed to behave according to their documented security properties.

## Out of scope

### Semantic correctness

The kernel does not determine whether an identity, boundary, law, or requested transition is desirable or correct in the external world.

A caller can construct a bad law and the kernel can faithfully enforce that bad law.

### External identity

Identity is application-supplied data.

The kernel does not authenticate a human, machine, service, or AI agent represented by that value.

### Key compromise

Possession of the signing private key allows an attacker to create valid signatures.

The kernel does not provide hardware-backed key custody, key rotation, revocation, threshold signing, or multi-signature authorization.

### Host compromise

The kernel does not defend against an attacker who controls the process or machine while trusted operations are being executed.

### Replay and rollback

A correctly signed persistence artifact can be presented again later.

Version 2 does not include timestamps, monotonic counters, replay caches, external ledgers, or freshness proofs.

The kernel therefore does not establish that a valid signed artifact is the newest valid artifact.

### Unsigned persistence authenticity

Unsigned persistence performs decoding and internal consistency verification.

It is not an authenticity mechanism.

An attacker capable of replacing a complete unsigned artifact and recomputing its internal structures is outside the protection boundary of unsigned persistence.

Use signed persistence when authenticity is required.

### Availability

The kernel does not provide denial-of-service protection.

### Confidentiality

Persistence is not encrypted.

The kernel provides no confidentiality guarantee for persisted state.

## Fingerprint boundary

RealityFingerprint and drift_check() serve different purposes.

The fingerprint incorporates identity, birth boundary, birth law, state, initial state, and current movement-memory hash.

Active boundary and law drift are separately evaluated by drift_check().

A fingerprint should therefore not be treated as a complete standalone substitute for the integrity and drift reports.

## Persistence protocol v2

Unsigned persisted values include an explicit persistence version.

Signed artifacts include an explicit protocol version.

The supported version is:

```text
2
```

Version 2 uses Postcard serialization.

Version 1 used bincode.

The formats are intentionally not considered compatible.

## Security objective

The kernel's objective is narrow:

Make permitted state movement explicit, record that movement in tamper-evident memory, make continuity and drift inspectable, and allow persisted state to be authenticated when an external trusted signing key is available.

Claims beyond that boundary are not made.
