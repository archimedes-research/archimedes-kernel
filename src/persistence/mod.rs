use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::primitives::RealityWire;
use crate::verification::{MovementError, SnapshotValidationError};
use crate::{Reality, RealitySnapshot};

pub const PERSISTENCE_VERSION: u8 = 2;

const REALITY_DOMAIN: &[u8] = b"ARCHIMEDES-KERNEL-SIGNED-REALITY-V2";
const SNAPSHOT_DOMAIN: &[u8] = b"ARCHIMEDES-KERNEL-SIGNED-SNAPSHOT-V2";

#[derive(Debug)]
pub enum PersistenceError {
    Io(io::Error),
    Serialization(postcard::Error),
    IntegrityFailure(&'static str),
    SemanticValidity(MovementError),
    SnapshotValidity(SnapshotValidationError),
    Signature(&'static str),
    UnsupportedVersion { expected: u8, found: u8 },
    PublicKeyMismatch,
    TrailingData { bytes: usize },
    DurabilityIndeterminate(io::Error),
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistenceError::Io(error) => {
                write!(f, "IO error: {error}")
            }
            PersistenceError::Serialization(error) => {
                write!(f, "Serialization error: {error}")
            }
            PersistenceError::IntegrityFailure(message) => {
                write!(f, "Integrity failure: {message}")
            }
            PersistenceError::SemanticValidity(error) => {
                write!(f, "Reality semantic validity failure: {error}")
            }
            PersistenceError::SnapshotValidity(error) => {
                write!(f, "Snapshot validity failure: {error}")
            }
            PersistenceError::Signature(message) => {
                write!(f, "Signature failure: {message}")
            }
            PersistenceError::UnsupportedVersion { expected, found } => {
                write!(
                    f,
                    "Unsupported persistence version: expected {expected}, found {found}"
                )
            }
            PersistenceError::PublicKeyMismatch => {
                write!(
                    f,
                    "Embedded public key does not match expected authority key"
                )
            }
            PersistenceError::TrailingData { bytes } => {
                write!(
                    f,
                    "Persistence artifact contains {bytes} unexplained trailing bytes"
                )
            }
            PersistenceError::DurabilityIndeterminate(error) => {
                write!(
                    f,
                    "Destination replacement occurred but directory durability could not be confirmed: {error}"
                )
            }
        }
    }
}

impl std::error::Error for PersistenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PersistenceError::Io(error) => Some(error),
            PersistenceError::SemanticValidity(error) => Some(error),
            PersistenceError::SnapshotValidity(error) => Some(error),
            PersistenceError::DurabilityIndeterminate(error) => Some(error),
            PersistenceError::Serialization(_)
            | PersistenceError::IntegrityFailure(_)
            | PersistenceError::Signature(_)
            | PersistenceError::UnsupportedVersion { .. }
            | PersistenceError::PublicKeyMismatch
            | PersistenceError::TrailingData { .. } => None,
        }
    }
}

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn parent_directory(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn create_temp_sibling(path: &Path) -> Result<(PathBuf, File), PersistenceError> {
    let parent = parent_directory(path);

    let file_name = path.file_name().ok_or_else(|| {
        PersistenceError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "persistence destination has no file name",
        ))
    })?;

    for _ in 0..32 {
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        let temp_name = format!(
            ".{}.archimedes-tmp-{}-{timestamp}-{counter}",
            file_name.to_string_lossy(),
            std::process::id(),
        );

        let temp_path = parent.join(temp_name);

        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(file) => {
                return Ok((temp_path, file));
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                continue;
            }
            Err(error) => {
                return Err(PersistenceError::Io(error));
            }
        }
    }

    Err(PersistenceError::Io(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not allocate unique sibling persistence temporary file",
    )))
}

#[cfg(unix)]
fn sync_parent_after_rename(path: &Path) -> io::Result<()> {
    File::open(parent_directory(path))?.sync_all()
}

#[cfg(not(unix))]
fn sync_parent_after_rename(_path: &Path) -> io::Result<()> {
    Ok(())
}

fn write_bytes_crash_conscious_with<F>(
    path: &Path,
    bytes: &[u8],
    before_rename: F,
) -> Result<(), PersistenceError>
where
    F: FnOnce(&Path) -> io::Result<()>,
{
    let (temp_path, mut file) = create_temp_sibling(path)?;

    if let Err(error) = file.write_all(bytes) {
        drop(file);
        let _ = fs::remove_file(&temp_path);
        return Err(PersistenceError::Io(error));
    }

    if let Err(error) = file.sync_all() {
        drop(file);
        let _ = fs::remove_file(&temp_path);
        return Err(PersistenceError::Io(error));
    }

    drop(file);

    if let Err(error) = before_rename(&temp_path) {
        let _ = fs::remove_file(&temp_path);
        return Err(PersistenceError::Io(error));
    }

    if let Err(error) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(PersistenceError::Io(error));
    }

    sync_parent_after_rename(path).map_err(PersistenceError::DurabilityIndeterminate)
}

fn write_bytes_crash_conscious(path: &Path, bytes: &[u8]) -> Result<(), PersistenceError> {
    write_bytes_crash_conscious_with(path, bytes, |_| Ok(()))
}

fn write_postcard<T: Serialize>(path: &Path, value: &T) -> Result<(), PersistenceError> {
    let bytes = postcard::to_allocvec(value).map_err(PersistenceError::Serialization)?;

    write_bytes_crash_conscious(path, &bytes)
}

#[cfg(test)]
fn write_postcard_with_pre_rename_failure<T: Serialize>(
    path: &Path,
    value: &T,
) -> Result<(), PersistenceError> {
    let bytes = postcard::to_allocvec(value).map_err(PersistenceError::Serialization)?;

    write_bytes_crash_conscious_with(path, &bytes, |_| {
        Err(io::Error::other("injected pre-rename persistence failure"))
    })
}

fn read_postcard<T: DeserializeOwned>(path: &Path) -> Result<T, PersistenceError> {
    let bytes = fs::read(path).map_err(PersistenceError::Io)?;

    let (value, trailing) =
        postcard::take_from_bytes(&bytes).map_err(PersistenceError::Serialization)?;

    if !trailing.is_empty() {
        return Err(PersistenceError::TrailingData {
            bytes: trailing.len(),
        });
    }

    Ok(value)
}

fn check_version(found: u8) -> Result<(), PersistenceError> {
    if found != PERSISTENCE_VERSION {
        return Err(PersistenceError::UnsupportedVersion {
            expected: PERSISTENCE_VERSION,
            found,
        });
    }

    Ok(())
}

fn validate_reality(reality: &Reality) -> Result<(), PersistenceError> {
    reality
        .validate()
        .map_err(PersistenceError::SemanticValidity)
}

fn validate_snapshot(snapshot: &RealitySnapshot) -> Result<(), PersistenceError> {
    snapshot
        .validate()
        .map_err(PersistenceError::SnapshotValidity)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PersistedReality {
    persistence_version: u8,
    reality: Reality,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PersistedSnapshot {
    persistence_version: u8,
    snapshot: RealitySnapshot,
}

pub fn save_reality(reality: &Reality, path: &Path) -> Result<(), PersistenceError> {
    validate_reality(reality)?;

    let persisted = PersistedReality {
        persistence_version: PERSISTENCE_VERSION,
        reality: reality.clone(),
    };

    write_postcard(path, &persisted)
}

pub fn load_reality(path: &Path) -> Result<Reality, PersistenceError> {
    let persisted: PersistedReality = read_postcard(path)?;

    check_version(persisted.persistence_version)?;

    let reality = persisted.reality;

    validate_reality(&reality)?;

    Ok(reality)
}

pub fn save_snapshot(snapshot: &RealitySnapshot, path: &Path) -> Result<(), PersistenceError> {
    validate_snapshot(snapshot)?;

    let persisted = PersistedSnapshot {
        persistence_version: PERSISTENCE_VERSION,
        snapshot: snapshot.clone(),
    };

    write_postcard(path, &persisted)
}

pub fn load_snapshot(path: &Path) -> Result<RealitySnapshot, PersistenceError> {
    let persisted: PersistedSnapshot = read_postcard(path)?;

    check_version(persisted.persistence_version)?;

    validate_snapshot(&persisted.snapshot)?;

    Ok(persisted.snapshot)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedReality {
    pub protocol_version: u8,
    pub reality: Reality,
    pub public_key: [u8; 32],
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedSnapshot {
    pub protocol_version: u8,
    pub snapshot: RealitySnapshot,
    pub public_key: [u8; 32],
    pub signature: Vec<u8>,
}

#[derive(Serialize)]
struct SignableReality<'a> {
    domain: &'a [u8],
    protocol_version: u8,
    reality: &'a Reality,
    public_key: [u8; 32],
}

#[derive(Serialize)]
struct SignableSnapshot<'a> {
    domain: &'a [u8],
    protocol_version: u8,
    snapshot: &'a RealitySnapshot,
    public_key: [u8; 32],
}

fn sign_bytes(message: &[u8], signing_key: &SigningKey) -> ([u8; 32], [u8; 64]) {
    let signature: Signature = signing_key.sign(message);

    (signing_key.verifying_key().to_bytes(), signature.to_bytes())
}

fn verify_bytes(
    message: &[u8],
    public_key: &[u8; 32],
    signature: &[u8],
) -> Result<(), PersistenceError> {
    let sig_bytes: [u8; 64] = signature
        .try_into()
        .map_err(|_| PersistenceError::Signature("invalid signature length"))?;

    let verifying_key = VerifyingKey::from_bytes(public_key)
        .map_err(|_| PersistenceError::Signature("invalid public key"))?;

    let signature = Signature::from_bytes(&sig_bytes);

    verifying_key
        .verify_strict(message, &signature)
        .map_err(|_| PersistenceError::Signature("invalid signature"))
}

pub fn sign_reality(
    reality: &Reality,
    signing_key: &SigningKey,
) -> Result<SignedReality, PersistenceError> {
    validate_reality(reality)?;

    let public_key = signing_key.verifying_key().to_bytes();

    let signable = SignableReality {
        domain: REALITY_DOMAIN,
        protocol_version: PERSISTENCE_VERSION,
        reality,
        public_key,
    };

    let message = postcard::to_allocvec(&signable).map_err(PersistenceError::Serialization)?;

    let (_, signature) = sign_bytes(&message, signing_key);

    Ok(SignedReality {
        protocol_version: PERSISTENCE_VERSION,
        reality: reality.clone(),
        public_key,
        signature: signature.to_vec(),
    })
}

pub fn save_signed_reality(signed: &SignedReality, path: &Path) -> Result<(), PersistenceError> {
    check_version(signed.protocol_version)?;

    validate_reality(&signed.reality)?;

    let signable = SignableReality {
        domain: REALITY_DOMAIN,
        protocol_version: signed.protocol_version,
        reality: &signed.reality,
        public_key: signed.public_key,
    };

    let message = postcard::to_allocvec(&signable).map_err(PersistenceError::Serialization)?;

    verify_bytes(&message, &signed.public_key, &signed.signature)?;

    write_postcard(path, signed)
}

#[derive(Deserialize)]
struct SignedRealityWire {
    protocol_version: u8,
    reality: RealityWire,
    public_key: [u8; 32],
    signature: Vec<u8>,
}

#[derive(Serialize)]
struct SignableRealityWire<'a> {
    domain: &'a [u8],
    protocol_version: u8,
    reality: &'a RealityWire,
    public_key: [u8; 32],
}

pub fn load_signed_reality(
    path: &Path,
    expected_public_key: &[u8; 32],
) -> Result<SignedReality, PersistenceError> {
    let signed: SignedRealityWire = read_postcard(path)?;

    check_version(signed.protocol_version)?;

    if signed.public_key != *expected_public_key {
        return Err(PersistenceError::PublicKeyMismatch);
    }

    let signable = SignableRealityWire {
        domain: REALITY_DOMAIN,
        protocol_version: signed.protocol_version,
        reality: &signed.reality,
        public_key: signed.public_key,
    };

    let message = postcard::to_allocvec(&signable).map_err(PersistenceError::Serialization)?;

    verify_bytes(&message, expected_public_key, &signed.signature)?;

    let reality = signed
        .reality
        .into_validated()
        .map_err(PersistenceError::SemanticValidity)?;

    Ok(SignedReality {
        protocol_version: signed.protocol_version,
        reality,
        public_key: signed.public_key,
        signature: signed.signature,
    })
}

pub fn sign_snapshot(
    snapshot: &RealitySnapshot,
    signing_key: &SigningKey,
) -> Result<SignedSnapshot, PersistenceError> {
    validate_snapshot(snapshot)?;

    let public_key = signing_key.verifying_key().to_bytes();

    let signable = SignableSnapshot {
        domain: SNAPSHOT_DOMAIN,
        protocol_version: PERSISTENCE_VERSION,
        snapshot,
        public_key,
    };

    let message = postcard::to_allocvec(&signable).map_err(PersistenceError::Serialization)?;

    let (_, signature) = sign_bytes(&message, signing_key);

    Ok(SignedSnapshot {
        protocol_version: PERSISTENCE_VERSION,
        snapshot: snapshot.clone(),
        public_key,
        signature: signature.to_vec(),
    })
}

pub fn save_signed_snapshot(signed: &SignedSnapshot, path: &Path) -> Result<(), PersistenceError> {
    check_version(signed.protocol_version)?;

    validate_snapshot(&signed.snapshot)?;

    let signable = SignableSnapshot {
        domain: SNAPSHOT_DOMAIN,
        protocol_version: signed.protocol_version,
        snapshot: &signed.snapshot,
        public_key: signed.public_key,
    };

    let message = postcard::to_allocvec(&signable).map_err(PersistenceError::Serialization)?;

    verify_bytes(&message, &signed.public_key, &signed.signature)?;

    write_postcard(path, signed)
}

pub fn load_signed_snapshot(
    path: &Path,
    expected_public_key: &[u8; 32],
) -> Result<SignedSnapshot, PersistenceError> {
    let signed: SignedSnapshot = read_postcard(path)?;

    check_version(signed.protocol_version)?;

    if signed.public_key != *expected_public_key {
        return Err(PersistenceError::PublicKeyMismatch);
    }

    let signable = SignableSnapshot {
        domain: SNAPSHOT_DOMAIN,
        protocol_version: signed.protocol_version,
        snapshot: &signed.snapshot,
        public_key: signed.public_key,
    };

    let message = postcard::to_allocvec(&signable).map_err(PersistenceError::Serialization)?;

    verify_bytes(&message, expected_public_key, &signed.signature)?;

    validate_snapshot(&signed.snapshot)?;

    Ok(signed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{perform_movement_sequence, Boundary, Event, Identity, Law, State};
    use rand::rngs::OsRng;

    fn test_reality() -> Reality {
        Reality::new(
            Identity("persist-test".to_string()),
            Boundary {
                allowed_values: vec![
                    "before".to_string(),
                    "after".to_string(),
                    "done".to_string(),
                ],
            },
            Law {
                allowed_transitions: vec![
                    ("before".to_string(), "after".to_string()),
                    ("after".to_string(), "done".to_string()),
                ],
            },
            State {
                field: "before".to_string(),
            },
        )
        .expect("valid Reality construction")
    }

    fn moved_reality() -> Reality {
        let mut reality = test_reality();

        let events = vec![
            Event {
                proposed_field: "after".to_string(),
            },
            Event {
                proposed_field: "done".to_string(),
            },
        ];

        perform_movement_sequence(&mut reality, events).unwrap();

        reality
    }

    #[test]
    fn reality_roundtrip_preserves_integrity() {
        let reality = moved_reality();

        let path = std::env::temp_dir().join("archimedes-v2-reality.bin");

        save_reality(&reality, &path).unwrap();

        let loaded = load_reality(&path).unwrap();

        assert_eq!(reality, loaded);
        assert!(loaded.memory_integrity());
        assert!(!loaded.drift_check().hidden_drift_required);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn snapshot_roundtrip_matches_original() {
        let reality = test_reality();
        let snapshot = reality.snapshot();

        let path = std::env::temp_dir().join("archimedes-v2-snapshot.bin");

        save_snapshot(&snapshot, &path).unwrap();

        let loaded = load_snapshot(&path).unwrap();

        assert_eq!(snapshot, loaded);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn tampered_reality_file_is_rejected() {
        let reality = moved_reality();

        let path = std::env::temp_dir().join("archimedes-v2-tampered.bin");

        save_reality(&reality, &path).unwrap();

        let mut bytes = std::fs::read(&path).unwrap();
        let mid = bytes.len() / 2;
        bytes[mid] ^= 0xFF;
        std::fs::write(&path, &bytes).unwrap();

        assert!(load_reality(&path).is_err());

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn unsupported_unsigned_version_is_rejected() {
        let persisted = PersistedReality {
            persistence_version: 99,
            reality: test_reality(),
        };

        let path = std::env::temp_dir().join("archimedes-v2-version.bin");

        write_postcard(&path, &persisted).unwrap();

        let result = load_reality(&path);

        assert!(matches!(
            result,
            Err(PersistenceError::UnsupportedVersion { .. })
        ));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn signed_reality_roundtrip_works() {
        let reality = moved_reality();

        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let public_key = signing_key.verifying_key().to_bytes();

        let signed = sign_reality(&reality, &signing_key).unwrap();

        let path = std::env::temp_dir().join("archimedes-v2-signed-reality.bin");

        save_signed_reality(&signed, &path).unwrap();

        let loaded = load_signed_reality(&path, &public_key).unwrap();

        assert_eq!(loaded.protocol_version, PERSISTENCE_VERSION);
        assert_eq!(loaded.reality, reality);
        assert_eq!(loaded.public_key, public_key);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn signed_reality_rejects_wrong_key() {
        let reality = moved_reality();

        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let wrong_key = SigningKey::generate(&mut csprng).verifying_key().to_bytes();

        let signed = sign_reality(&reality, &signing_key).unwrap();

        let path = std::env::temp_dir().join("archimedes-v2-wrong-key.bin");

        save_signed_reality(&signed, &path).unwrap();

        assert!(load_signed_reality(&path, &wrong_key).is_err());

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn signed_reality_rejects_tampered_embedded_key() {
        let reality = moved_reality();

        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let public_key = signing_key.verifying_key().to_bytes();

        let mut signed = sign_reality(&reality, &signing_key).unwrap();
        signed.public_key[0] ^= 0x01;

        let path = std::env::temp_dir().join("archimedes-v2-key-tamper.bin");

        write_postcard(&path, &signed).unwrap();

        let result = load_signed_reality(&path, &public_key);

        assert!(matches!(result, Err(PersistenceError::PublicKeyMismatch)));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn signed_reality_rejects_tampered_payload() {
        let reality = moved_reality();

        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let public_key = signing_key.verifying_key().to_bytes();

        let mut signed = sign_reality(&reality, &signing_key).unwrap();
        signed.reality.state.field = "tampered".to_string();

        let path = std::env::temp_dir().join("archimedes-v2-payload-tamper.bin");

        write_postcard(&path, &signed).unwrap();

        let result = load_signed_reality(&path, &public_key);

        assert!(matches!(result, Err(PersistenceError::Signature(_))));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn unsupported_signed_version_is_rejected() {
        let reality = moved_reality();

        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let public_key = signing_key.verifying_key().to_bytes();

        let mut signed = sign_reality(&reality, &signing_key).unwrap();
        signed.protocol_version = 99;

        let path = std::env::temp_dir().join("archimedes-v2-protocol-version.bin");

        write_postcard(&path, &signed).unwrap();

        let result = load_signed_reality(&path, &public_key);

        assert!(matches!(
            result,
            Err(PersistenceError::UnsupportedVersion { .. })
        ));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn signed_snapshot_roundtrip_works() {
        let reality = test_reality();
        let snapshot = reality.snapshot();

        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let public_key = signing_key.verifying_key().to_bytes();

        let signed = sign_snapshot(&snapshot, &signing_key).unwrap();

        let path = std::env::temp_dir().join("archimedes-v2-signed-snapshot.bin");

        save_signed_snapshot(&signed, &path).unwrap();

        let loaded = load_signed_snapshot(&path, &public_key).unwrap();

        assert_eq!(loaded.protocol_version, PERSISTENCE_VERSION);
        assert_eq!(loaded.snapshot, snapshot);
        assert_eq!(loaded.public_key, public_key);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn signed_snapshot_rejects_tampered_embedded_key() {
        let reality = test_reality();
        let snapshot = reality.snapshot();

        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let public_key = signing_key.verifying_key().to_bytes();

        let mut signed = sign_snapshot(&snapshot, &signing_key).unwrap();
        signed.public_key[0] ^= 0x01;

        let path = std::env::temp_dir().join("archimedes-v2-snapshot-key-tamper.bin");

        write_postcard(&path, &signed).unwrap();

        let result = load_signed_snapshot(&path, &public_key);

        assert!(matches!(result, Err(PersistenceError::PublicKeyMismatch)));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn signed_snapshot_rejects_tampered_payload() {
        let reality = test_reality();
        let snapshot = reality.snapshot();

        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let public_key = signing_key.verifying_key().to_bytes();

        let mut signed = sign_snapshot(&snapshot, &signing_key).unwrap();
        signed.snapshot.state.field = "tampered".to_string();

        let path = std::env::temp_dir().join("archimedes-v2-snapshot-tamper.bin");

        write_postcard(&path, &signed).unwrap();

        let result = load_signed_snapshot(&path, &public_key);

        assert!(matches!(result, Err(PersistenceError::Signature(_))));

        let _ = std::fs::remove_file(&path);
    }
    #[test]
    fn invalid_reality_is_rejected_before_save_or_sign() {
        let mut reality = test_reality();
        reality.identity.0.clear();

        let path = std::env::temp_dir().join(format!(
            "archimedes-v2-invalid-save-{}.bin",
            std::process::id(),
        ));

        let save_result = save_reality(&reality, &path);

        assert!(matches!(
            save_result,
            Err(PersistenceError::SemanticValidity(_))
        ));

        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);

        let sign_result = sign_reality(&reality, &signing_key);

        assert!(matches!(
            sign_result,
            Err(PersistenceError::SemanticValidity(_))
        ));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn injected_pre_rename_failure_preserves_target_and_cleans_temp() {
        let path = std::env::temp_dir().join(format!(
            "archimedes-v2-pre-rename-{}.bin",
            std::process::id(),
        ));

        std::fs::write(&path, b"ARCHIMEDES-SENTINEL").unwrap();

        let persisted = PersistedReality {
            persistence_version: PERSISTENCE_VERSION,
            reality: test_reality(),
        };

        let result = write_postcard_with_pre_rename_failure(&path, &persisted);

        assert!(matches!(result, Err(PersistenceError::Io(_))));

        assert_eq!(std::fs::read(&path).unwrap(), b"ARCHIMEDES-SENTINEL",);

        let file_name = path.file_name().unwrap().to_string_lossy();

        let prefix = format!(".{file_name}.archimedes-tmp-");

        let residue_exists = std::fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .any(|entry| entry.file_name().to_string_lossy().starts_with(&prefix));

        assert!(!residue_exists);

        let _ = std::fs::remove_file(path);
    }
}
