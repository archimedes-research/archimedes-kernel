# Security Policy

## Supported versions

The actively supported release line is:

```text
2.x
```

The v1 persistence format is superseded and is not part of the current security-maintenance line.

## Reporting a vulnerability

Please report suspected security vulnerabilities privately to:

`archimedes-research@proton.me`

Do not open a public GitHub issue for a vulnerability that has not yet been disclosed.

Include, where possible:

- affected version
- affected component
- reproduction steps
- security impact
- proof-of-concept input or code
- suggested mitigation

Do not include private keys, access tokens, credentials, or unrelated sensitive information.

## Scope

Security-relevant areas include:

- movement integrity
- hash-chain verification
- drift detection
- replay verification
- persistence decoding
- signed persistence
- Ed25519 verification
- protocol-version handling
- authority-key binding

## Disclosure

ARCHIMEDES will evaluate reported issues and publish corrections when evidence supports the finding.

No response-time or remediation-time SLA is promised.

## Security boundary

Before reporting behavior as a vulnerability, review:

`THREAT_MODEL.md`

Some behaviors—including replay of an otherwise valid signed artifact, lack of confidentiality, external identity verification, and key revocation—are currently documented limitations rather than implemented security guarantees.

## ARCH-005 3.0.0 qualification delta

The current Engineering candidate separates cryptographic authenticity from semantic validity.

A signed Reality is accepted by the authenticated loader only when:

1. the persistence protocol version is supported;
2. the embedded public key matches the externally supplied expected public key;
3. the Ed25519 signature is valid for the neutral persisted wire representation; and
4. the authenticated Reality satisfies the kernel semantic validity contract.

An invalid signature is reported as an authentication failure.

A correctly authenticated but semantically invalid Reality is reported as a semantic-validity failure.

Snapshot signing and loading likewise require snapshot self-consistency in addition to cryptographic checks.

Persistence framing rejects unexplained trailing bytes.

Local replacement uses a sibling temporary file, complete write, file synchronization, rename, and Unix parent-directory synchronization. A post-rename directory-sync error is explicitly treated as durability indeterminate because replacement may already have occurred.

These controls do not establish signer trust, secure private-key custody, host integrity, freshness, rollback prevention, confidentiality, distributed atomicity, or remote durability.

The crate candidate is version `3.0.0`; persistence protocol version `2` remains intentionally separate.

Independent Assurance of the exact candidate has not yet occurred.
