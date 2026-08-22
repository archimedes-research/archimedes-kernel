# Review-Ready Repository Structure Object v0

## Status

Accepted for Minimum Kernel Footing v0.  
Updated after cryptographic hashing (SHA‑256), unsigned persistence, and authenticated persistence using Ed25519 signatures.

## Repository Layout

archimedes-kernel/
├── Cargo.toml
├── Cargo.lock
├── LICENSE
├── README.md
├── rust-toolchain.toml
├── .gitignore
├── docs/
│ ├── repository-boundary-classification-v0.md
│ ├── review-ready-repository-structure-v0.md
│ ├── hostile-audit-packet-v0.md
│ ├── hostile-audit-report-v0.md
│ ├── ... previous reports ...
│ ├── hostile-audit-report-v0.11.md
│ ├── hostile-audit-report-v0.12.md
│ ├── certified-truth-record-minimum-kernel-footing-v0.md
│ ├── ... previous records ...
│ ├── certified-truth-record-minimum-kernel-footing-v0.11.md
│ └── certified-truth-record-minimum-kernel-footing-v0.12.md
├── examples/
│ └── demo.rs
└── src/
├── lib.rs
├── primitives/
│ └── mod.rs
├── movement/
│ └── mod.rs
├── verification/
│ └── mod.rs
└── persistence/
└── mod.rs


## Test Evidence

- Run `cargo test`
- All tests pass (currently 55 tests)
- No warnings
- Example binary runs successfully: `cargo run --example demo`

## Module Grouping Compliance

- `primitives/` contains only primitive types.
- `movement/` contains movement types and SHA‑256 hashing.
- `verification/` contains read-only verification types and functions.
- `persistence/` contains binary save/load and Ed25519 signed save/load functions.

## Boundary Compliance

- Dependencies: `sha2`, `serde`, `bincode`, `ed25519-dalek`, `rand`
- No production features.
- No domain-specific logic.
- `Reality` is encapsulated.

## Example Binary

- `examples/demo.rs` demonstrates full movement, verification, snapshot, unsigned persistence, and signed persistence.

## Certification Status

This structure is accepted as the lawful host boundary for the minimum kernel.
