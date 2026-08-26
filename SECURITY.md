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
