use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

use archimedes_kernel::{
    load_reality, load_signed_reality, load_signed_snapshot, load_snapshot, perform_movement,
    save_reality, save_signed_reality, save_signed_snapshot, save_snapshot, sign_reality,
    sign_snapshot, Boundary, Event, Identity, Law, MovementMemory, PersistenceError, Reality,
    RealitySnapshot, SignedSnapshot, State, PERSISTENCE_VERSION,
};

use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use serde::Serialize;

const SNAPSHOT_DOMAIN: &[u8] = b"ARCHIMEDES-KERNEL-SIGNED-SNAPSHOT-V2";

fn s(value: &str) -> String {
    value.to_string()
}

fn path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("arch005-eng03-{}-{label}.bin", std::process::id(),))
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
}

fn valid_reality() -> Reality {
    Reality::new(
        Identity(s("eng03")),
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

fn moved_reality() -> Reality {
    let mut reality = valid_reality();

    perform_movement(
        &mut reality,
        Event {
            proposed_field: s("b"),
        },
    )
    .expect("lawful movement");

    reality
}

#[derive(Serialize)]
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
struct SignableSnapshot<'a> {
    domain: &'a [u8],
    protocol_version: u8,
    snapshot: &'a RealitySnapshot,
    public_key: [u8; 32],
}

fn invalid_reality_wire() -> RealityWire {
    RealityWire {
        identity: Identity(String::new()),
        boundary: Boundary {
            allowed_values: vec![s("inside")],
        },
        law: Law {
            allowed_transitions: vec![(s("inside"), s("inside"))],
        },
        state: State { field: s("inside") },
        initial_state: State { field: s("inside") },
        memory: MovementMemory::default(),
        birth_boundary: Boundary {
            allowed_values: vec![s("inside")],
        },
        birth_law: Law {
            allowed_transitions: vec![(s("inside"), s("inside"))],
        },
    }
}

#[test]
fn invalid_reality_wire_is_rejected_by_public_deserialize() {
    let bytes = postcard::to_allocvec(&invalid_reality_wire()).expect("wire serialization");

    assert!(postcard::from_bytes::<Reality>(&bytes,).is_err());
}

#[test]
fn invalid_reality_artifact_is_rejected_on_load() {
    let persisted = PersistedRealityWire {
        persistence_version: PERSISTENCE_VERSION,
        reality: invalid_reality_wire(),
    };

    let bytes = postcard::to_allocvec(&persisted).expect("artifact serialization");

    let target = path("invalid-reality");
    cleanup(&target);

    fs::write(&target, bytes).expect("write adversarial artifact");

    assert!(load_reality(&target).is_err());

    cleanup(&target);
}

#[test]
fn inconsistent_snapshot_is_rejected_by_save_and_sign() {
    let reality = moved_reality();
    let mut snapshot = reality.snapshot();

    snapshot.state.field = s("tampered");

    assert!(snapshot.validate().is_err());

    let target = path("invalid-snapshot-save");
    cleanup(&target);

    assert!(save_snapshot(&snapshot, &target,).is_err());

    let mut rng = OsRng;
    let key = SigningKey::generate(&mut rng);

    assert!(sign_snapshot(&snapshot, &key,).is_err());

    cleanup(&target);
}

#[test]
fn authenticated_inconsistent_snapshot_is_rejected_on_load() {
    let reality = moved_reality();
    let mut snapshot = reality.snapshot();

    snapshot.state.field = s("tampered");

    let mut rng = OsRng;
    let key = SigningKey::generate(&mut rng);

    let public_key = key.verifying_key().to_bytes();

    let signable = SignableSnapshot {
        domain: SNAPSHOT_DOMAIN,
        protocol_version: PERSISTENCE_VERSION,
        snapshot: &snapshot,
        public_key,
    };

    let message = postcard::to_allocvec(&signable).expect("signable serialization");

    let signature = key.sign(&message).to_bytes().to_vec();

    let signed = SignedSnapshot {
        protocol_version: PERSISTENCE_VERSION,
        snapshot,
        public_key,
        signature,
    };

    let bytes = postcard::to_allocvec(&signed).expect("signed artifact serialization");

    let target = path("authenticated-invalid-snapshot");

    cleanup(&target);

    fs::write(&target, bytes).expect("write signed adversarial artifact");

    let result = load_signed_snapshot(&target, &public_key);

    assert!(matches!(result, Err(PersistenceError::SnapshotValidity(_))));

    cleanup(&target);
}

#[test]
fn trailing_bytes_are_rejected() {
    let reality = moved_reality();
    let target = path("trailing");

    cleanup(&target);

    save_reality(&reality, &target).expect("save valid Reality");

    let mut file = OpenOptions::new()
        .append(true)
        .open(&target)
        .expect("open artifact for adversarial append");

    file.write_all(&[0xde, 0xad, 0xbe, 0xef])
        .expect("append trailing bytes");

    file.sync_all().expect("sync adversarial append");

    let result = load_reality(&target);

    assert!(matches!(
        result,
        Err(PersistenceError::TrailingData { bytes: 4 })
    ));

    cleanup(&target);
}

#[cfg(unix)]
#[test]
fn replacement_uses_new_inode_and_leaves_no_temp_residue() {
    let reality = moved_reality();

    let target = path("replacement");
    cleanup(&target);

    fs::write(&target, b"ARCHIMEDES-SENTINEL").expect("write prior target");

    let before_inode = fs::metadata(&target).expect("before metadata").ino();

    save_reality(&reality, &target).expect("crash-conscious replacement");

    let after_inode = fs::metadata(&target).expect("after metadata").ino();

    assert_ne!(before_inode, after_inode,);

    assert_eq!(load_reality(&target).expect("replacement loads"), reality,);

    let file_name = target.file_name().unwrap().to_string_lossy();

    let prefix = format!(".{file_name}.archimedes-tmp-");

    let residue_exists = fs::read_dir(target.parent().unwrap())
        .expect("read target directory")
        .filter_map(Result::ok)
        .any(|entry| entry.file_name().to_string_lossy().starts_with(&prefix));

    assert!(!residue_exists);

    cleanup(&target);
}

#[test]
fn valid_unsigned_and_signed_roundtrips_remain_valid() {
    let reality = moved_reality();
    let snapshot = reality.snapshot();

    assert_eq!(reality.validate(), Ok(()));
    assert_eq!(snapshot.validate(), Ok(()));

    let reality_path = path("valid-reality");

    let snapshot_path = path("valid-snapshot");

    let signed_reality_path = path("valid-signed-reality");

    let signed_snapshot_path = path("valid-signed-snapshot");

    for target in [
        &reality_path,
        &snapshot_path,
        &signed_reality_path,
        &signed_snapshot_path,
    ] {
        cleanup(target);
    }

    save_reality(&reality, &reality_path).expect("save Reality");

    assert_eq!(load_reality(&reality_path).expect("load Reality"), reality,);

    save_snapshot(&snapshot, &snapshot_path).expect("save Snapshot");

    assert_eq!(
        load_snapshot(&snapshot_path).expect("load Snapshot"),
        snapshot,
    );

    let mut rng = OsRng;
    let key = SigningKey::generate(&mut rng);

    let public_key = key.verifying_key().to_bytes();

    let signed_reality = sign_reality(&reality, &key).expect("sign Reality");

    save_signed_reality(&signed_reality, &signed_reality_path).expect("save signed Reality");

    let loaded_signed_reality =
        load_signed_reality(&signed_reality_path, &public_key).expect("load signed Reality");

    assert_eq!(loaded_signed_reality.reality, reality,);

    let signed_snapshot = sign_snapshot(&snapshot, &key).expect("sign Snapshot");

    save_signed_snapshot(&signed_snapshot, &signed_snapshot_path).expect("save signed Snapshot");

    let loaded_signed_snapshot =
        load_signed_snapshot(&signed_snapshot_path, &public_key).expect("load signed Snapshot");

    assert_eq!(loaded_signed_snapshot.snapshot, snapshot,);

    for target in [
        &reality_path,
        &snapshot_path,
        &signed_reality_path,
        &signed_snapshot_path,
    ] {
        cleanup(target);
    }
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

#[test]
fn authenticated_invalid_reality_is_semantic_failure_after_authentication() {
    const REALITY_DOMAIN: &[u8] = b"ARCHIMEDES-KERNEL-SIGNED-REALITY-V2";

    let mut rng = OsRng;

    let key = SigningKey::generate(&mut rng);

    let public_key = key.verifying_key().to_bytes();

    let reality = invalid_reality_wire();

    let signable = SignableRealityWire {
        domain: REALITY_DOMAIN,
        protocol_version: PERSISTENCE_VERSION,
        reality: &reality,
        public_key,
    };

    let message =
        postcard::to_allocvec(&signable).expect("invalid Reality wire must remain serializable");

    let signature = key.sign(&message).to_bytes().to_vec();

    let signed = SignedRealityWire {
        protocol_version: PERSISTENCE_VERSION,
        reality,
        public_key,
        signature,
    };

    let bytes = postcard::to_allocvec(&signed)
        .expect("authenticated invalid Reality artifact serialization");

    let target = path("authenticated-invalid-reality");

    cleanup(&target);

    fs::write(&target, bytes).expect("write authenticated invalid Reality");

    let result = load_signed_reality(&target, &public_key);

    assert!(matches!(result, Err(PersistenceError::SemanticValidity(_))));

    cleanup(&target);
}
