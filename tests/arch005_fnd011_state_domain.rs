use std::fs;
use std::path::{Path, PathBuf};

use archimedes_kernel::{
    load_reality, load_signed_reality, load_signed_snapshot, save_snapshot, sign_snapshot,
    Boundary, Event, Identity, Law, MovementError, MovementMemory, PersistenceError, Reality,
    RealitySnapshot, SignedSnapshot, SnapshotValidationError, State, PERSISTENCE_VERSION,
};
use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use serde::Serialize;

const REALITY_DOMAIN: &[u8] = b"ARCHIMEDES-KERNEL-SIGNED-REALITY-V2";
const SNAPSHOT_DOMAIN: &[u8] = b"ARCHIMEDES-KERNEL-SIGNED-SNAPSHOT-V2";

fn s(value: &str) -> String {
    value.to_string()
}

fn path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("arch005-fnd011-{}-{label}.bin", std::process::id(),))
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
}

fn valid_reality() -> Reality {
    Reality::new(
        Identity(s("fnd011")),
        Boundary {
            allowed_values: vec![s("a"), s("b")],
        },
        Law {
            allowed_transitions: vec![(s("a"), s("b"))],
        },
        State { field: s("a") },
    )
    .expect("valid Reality")
}

#[derive(Clone, Serialize)]
struct RealityWire {
    identity: Identity,
    boundary: Boundary,
    law: Law,
    state: State,
    initial_state: State,
    memory: MovementMemory,
    birth_boundary: Boundary,
    birth_law: Law,
}

#[derive(Serialize)]
struct PersistedRealityWire {
    persistence_version: u8,
    reality: RealityWire,
}

#[derive(Serialize)]
struct SignableRealityWire<'a> {
    domain: &'a [u8],
    protocol_version: u8,
    reality: &'a RealityWire,
    public_key: [u8; 32],
}

#[derive(Serialize)]
struct SignedRealityWire {
    protocol_version: u8,
    reality: RealityWire,
    public_key: [u8; 32],
    signature: Vec<u8>,
}

#[derive(Serialize)]
struct SignableSnapshot<'a> {
    domain: &'a [u8],
    protocol_version: u8,
    snapshot: &'a RealitySnapshot,
    public_key: [u8; 32],
}

fn valid_wire() -> RealityWire {
    RealityWire {
        identity: Identity(s("wire")),
        boundary: Boundary {
            allowed_values: vec![s("a"), s("b")],
        },
        law: Law {
            allowed_transitions: vec![(s("a"), s("b"))],
        },
        state: State { field: s("a") },
        initial_state: State { field: s("a") },
        memory: MovementMemory::default(),
        birth_boundary: Boundary {
            allowed_values: vec![s("a"), s("b")],
        },
        birth_law: Law {
            allowed_transitions: vec![(s("a"), s("b"))],
        },
    }
}

fn empty_domain_wire() -> RealityWire {
    let mut wire = valid_wire();

    wire.boundary.allowed_values.push(String::new());
    wire.birth_boundary.allowed_values.push(String::new());

    wire
}

#[test]
fn constructor_rejects_empty_boundary_member() {
    let result = Reality::new(
        Identity(s("boundary-empty")),
        Boundary {
            allowed_values: vec![s("a"), String::new()],
        },
        Law {
            allowed_transitions: vec![(s("a"), s("a"))],
        },
        State { field: s("a") },
    );

    assert_eq!(result, Err(MovementError::StateMissing));
}

#[test]
fn constructor_rejects_empty_law_endpoint() {
    let result = Reality::new(
        Identity(s("law-empty")),
        Boundary {
            allowed_values: vec![s("a"), String::new()],
        },
        Law {
            allowed_transitions: vec![(s("a"), String::new())],
        },
        State { field: s("a") },
    );

    assert_eq!(result, Err(MovementError::StateMissing));
}

#[test]
fn constructor_rejects_empty_state_value() {
    let result = Reality::new(
        Identity(s("state-empty")),
        Boundary {
            allowed_values: vec![String::new(), s("a")],
        },
        Law {
            allowed_transitions: vec![(String::new(), s("a"))],
        },
        State {
            field: String::new(),
        },
    );

    assert_eq!(result, Err(MovementError::StateMissing));
}

#[test]
fn law_check_rejects_empty_state_value_even_if_transition_declares_it() {
    let law = Law {
        allowed_transitions: vec![(s("a"), String::new())],
    };

    let current = State { field: s("a") };

    let event = Event {
        proposed_field: String::new(),
    };

    assert!(!law.check(&current, &event));
}

#[test]
fn public_deserialization_rejects_empty_birth_domain() {
    let mut boundary_wire = valid_wire();

    boundary_wire
        .birth_boundary
        .allowed_values
        .push(String::new());

    let bytes = postcard::to_allocvec(&boundary_wire).expect("serialize empty birth-boundary wire");

    assert!(postcard::from_bytes::<Reality>(&bytes).is_err());

    let mut law_wire = valid_wire();

    law_wire.birth_boundary.allowed_values.push(String::new());

    law_wire
        .birth_law
        .allowed_transitions
        .push((s("a"), String::new()));

    let bytes = postcard::to_allocvec(&law_wire).expect("serialize empty birth-law wire");

    assert!(postcard::from_bytes::<Reality>(&bytes).is_err());
}

#[test]
fn unsigned_persistence_rejects_empty_domain_artifact() {
    let persisted = PersistedRealityWire {
        persistence_version: PERSISTENCE_VERSION,
        reality: empty_domain_wire(),
    };

    let bytes = postcard::to_allocvec(&persisted).expect("serialize empty-domain Reality artifact");

    let target = path("unsigned-empty-domain");

    cleanup(&target);

    fs::write(&target, bytes).expect("write unsigned empty-domain artifact");

    assert!(load_reality(&target).is_err());

    cleanup(&target);
}

#[test]
fn authenticated_signed_reality_rejects_empty_domain_after_authentication() {
    let mut rng = OsRng;
    let key = SigningKey::generate(&mut rng);
    let public_key = key.verifying_key().to_bytes();

    let reality = empty_domain_wire();

    let signable = SignableRealityWire {
        domain: REALITY_DOMAIN,
        protocol_version: PERSISTENCE_VERSION,
        reality: &reality,
        public_key,
    };

    let message =
        postcard::to_allocvec(&signable).expect("serialize signable empty-domain Reality");

    let signature = key.sign(&message).to_bytes().to_vec();

    let signed = SignedRealityWire {
        protocol_version: PERSISTENCE_VERSION,
        reality,
        public_key,
        signature,
    };

    let bytes =
        postcard::to_allocvec(&signed).expect("serialize authenticated empty-domain Reality");

    let target = path("signed-empty-domain");

    cleanup(&target);

    fs::write(&target, bytes).expect("write authenticated empty-domain Reality");

    let result = load_signed_reality(&target, &public_key);

    assert!(matches!(
        result,
        Err(PersistenceError::SemanticValidity(
            MovementError::StateMissing
        ))
    ));

    cleanup(&target);
}

#[test]
fn snapshot_validation_rejects_empty_active_and_birth_domains_before_other_checks() {
    let base = valid_reality().snapshot();

    let mut active_boundary = base.clone();
    active_boundary.boundary.allowed_values.push(String::new());

    let mut birth_boundary = base.clone();
    birth_boundary
        .birth_boundary
        .allowed_values
        .push(String::new());

    let mut active_law = base.clone();
    active_law.boundary.allowed_values.push(String::new());
    active_law
        .law
        .allowed_transitions
        .push((s("a"), String::new()));

    let mut birth_law = base.clone();
    birth_law.birth_boundary.allowed_values.push(String::new());
    birth_law
        .birth_law
        .allowed_transitions
        .push((s("a"), String::new()));

    let mut rng = OsRng;
    let key = SigningKey::generate(&mut rng);

    for (label, snapshot) in [
        ("active-boundary", active_boundary),
        ("birth-boundary", birth_boundary),
        ("active-law", active_law),
        ("birth-law", birth_law),
    ] {
        assert_eq!(
            snapshot.validate(),
            Err(SnapshotValidationError::StateMissing),
            "{label}",
        );

        let target = path(label);
        cleanup(&target);

        assert!(
            matches!(
                save_snapshot(&snapshot, &target),
                Err(PersistenceError::SnapshotValidity(
                    SnapshotValidationError::StateMissing
                ))
            ),
            "{label}",
        );

        assert!(
            matches!(
                sign_snapshot(&snapshot, &key),
                Err(PersistenceError::SnapshotValidity(
                    SnapshotValidationError::StateMissing
                ))
            ),
            "{label}",
        );

        cleanup(&target);
    }
}

#[test]
fn authenticated_signed_snapshot_rejects_empty_domain_after_authentication() {
    let mut snapshot = valid_reality().snapshot();

    snapshot.birth_boundary.allowed_values.push(String::new());

    snapshot
        .birth_law
        .allowed_transitions
        .push((s("a"), String::new()));

    let mut rng = OsRng;
    let key = SigningKey::generate(&mut rng);
    let public_key = key.verifying_key().to_bytes();

    let signable = SignableSnapshot {
        domain: SNAPSHOT_DOMAIN,
        protocol_version: PERSISTENCE_VERSION,
        snapshot: &snapshot,
        public_key,
    };

    let message =
        postcard::to_allocvec(&signable).expect("serialize signable empty-domain Snapshot");

    let signature = key.sign(&message).to_bytes().to_vec();

    let signed = SignedSnapshot {
        protocol_version: PERSISTENCE_VERSION,
        snapshot,
        public_key,
        signature,
    };

    let bytes =
        postcard::to_allocvec(&signed).expect("serialize authenticated empty-domain Snapshot");

    let target = path("signed-snapshot-empty-domain");

    cleanup(&target);

    fs::write(&target, bytes).expect("write authenticated empty-domain Snapshot");

    let result = load_signed_snapshot(&target, &public_key);

    assert!(matches!(
        result,
        Err(PersistenceError::SnapshotValidity(
            SnapshotValidationError::StateMissing
        ))
    ));

    cleanup(&target);
}
