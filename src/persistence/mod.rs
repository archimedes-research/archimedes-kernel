use std::fs;
use std::io;
use std::path::Path;

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::{Reality, RealitySnapshot};

pub const PERSISTENCE_VERSION: u8 = 2;

const REALITY_DOMAIN: &[u8] = b"ARCHIMEDES-KERNEL-SIGNED-REALITY-V2";
const SNAPSHOT_DOMAIN: &[u8] = b"ARCHIMEDES-KERNEL-SIGNED-SNAPSHOT-V2";

#[derive(Debug)]
pub enum PersistenceError {
    Io(io::Error),
    Serialization(postcard::Error),
    IntegrityFailure(&'static str),
    Signature(&'static str),
    UnsupportedVersion { expected: u8, found: u8 },
    PublicKeyMismatch,
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistenceError::Io(e) => write!(f, "IO error: {e}"),
            PersistenceError::Serialization(e) => {
                write!(f, "Serialization error: {e}")
            }
            PersistenceError::IntegrityFailure(msg) => {
                write!(f, "Integrity failure: {msg}")
            }
            PersistenceError::Signature(msg) => {
                write!(f, "Signature failure: {msg}")
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
        }
    }
}

impl std::error::Error for PersistenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PersistenceError::Io(e) => Some(e),
            PersistenceError::Serialization(_)
            | PersistenceError::IntegrityFailure(_)
            | PersistenceError::Signature(_)
            | PersistenceError::UnsupportedVersion { .. }
            | PersistenceError::PublicKeyMismatch => None,
        }
    }
}

fn write_postcard<T: Serialize>(path: &Path, value: &T) -> Result<(), PersistenceError> {
    let bytes = postcard::to_allocvec(value).map_err(PersistenceError::Serialization)?;
    fs::write(path, bytes).map_err(PersistenceError::Io)
}

fn read_postcard<T: DeserializeOwned>(path: &Path) -> Result<T, PersistenceError> {
    let bytes = fs::read(path).map_err(PersistenceError::Io)?;
    postcard::from_bytes(&bytes).map_err(PersistenceError::Serialization)
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

    if !reality.memory_integrity() {
        return Err(PersistenceError::IntegrityFailure(
            "movement memory integrity failed",
        ));
    }

    if reality.drift_check().hidden_drift_required {
        return Err(PersistenceError::IntegrityFailure("hidden drift detected"));
    }

    Ok(reality)
}

pub fn save_snapshot(snapshot: &RealitySnapshot, path: &Path) -> Result<(), PersistenceError> {
    let persisted = PersistedSnapshot {
        persistence_version: PERSISTENCE_VERSION,
        snapshot: snapshot.clone(),
    };

    write_postcard(path, &persisted)
}

pub fn load_snapshot(path: &Path) -> Result<RealitySnapshot, PersistenceError> {
    let persisted: PersistedSnapshot = read_postcard(path)?;

    check_version(persisted.persistence_version)?;

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
    write_postcard(path, signed)
}

pub fn load_signed_reality(
    path: &Path,
    expected_public_key: &[u8; 32],
) -> Result<SignedReality, PersistenceError> {
    let signed: SignedReality = read_postcard(path)?;

    check_version(signed.protocol_version)?;

    if signed.public_key != *expected_public_key {
        return Err(PersistenceError::PublicKeyMismatch);
    }

    let signable = SignableReality {
        domain: REALITY_DOMAIN,
        protocol_version: signed.protocol_version,
        reality: &signed.reality,
        public_key: signed.public_key,
    };

    let message = postcard::to_allocvec(&signable).map_err(PersistenceError::Serialization)?;

    verify_bytes(&message, expected_public_key, &signed.signature)?;

    if !signed.reality.memory_integrity() {
        return Err(PersistenceError::IntegrityFailure(
            "movement memory integrity failed",
        ));
    }

    if signed.reality.drift_check().hidden_drift_required {
        return Err(PersistenceError::IntegrityFailure("hidden drift detected"));
    }

    Ok(signed)
}

pub fn sign_snapshot(
    snapshot: &RealitySnapshot,
    signing_key: &SigningKey,
) -> Result<SignedSnapshot, PersistenceError> {
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

        save_signed_reality(&signed, &path).unwrap();

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

        save_signed_reality(&signed, &path).unwrap();

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

        save_signed_reality(&signed, &path).unwrap();

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

        save_signed_snapshot(&signed, &path).unwrap();

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

        save_signed_snapshot(&signed, &path).unwrap();

        let result = load_signed_snapshot(&path, &public_key);

        assert!(matches!(result, Err(PersistenceError::Signature(_))));

        let _ = std::fs::remove_file(&path);
    }
}
