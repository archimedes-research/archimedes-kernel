use std::fs;
use std::io;
use std::path::Path;

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::{Reality, RealitySnapshot};

#[derive(Debug)]
pub enum PersistenceError {
    Io(io::Error),
    Serialization(bincode::Error),
    IntegrityFailure(&'static str),
    Signature(&'static str),
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistenceError::Io(e) => write!(f, "IO error: {}", e),
            PersistenceError::Serialization(e) => write!(f, "Serialization error: {}", e),
            PersistenceError::IntegrityFailure(msg) => write!(f, "Integrity failure: {}", msg),
            PersistenceError::Signature(msg) => write!(f, "Signature failure: {}", msg),
        }
    }
}

impl std::error::Error for PersistenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PersistenceError::Io(e) => Some(e),
            PersistenceError::Serialization(e) => Some(e),
            PersistenceError::IntegrityFailure(_) => None,
            PersistenceError::Signature(_) => None,
        }
    }
}

fn write_bincode<T: Serialize>(path: &Path, value: &T) -> Result<(), PersistenceError> {
    let bytes = bincode::serialize(value).map_err(PersistenceError::Serialization)?;
    fs::write(path, bytes).map_err(PersistenceError::Io)
}

fn read_bincode<T: DeserializeOwned>(path: &Path) -> Result<T, PersistenceError> {
    let bytes = fs::read(path).map_err(PersistenceError::Io)?;
    bincode::deserialize(&bytes).map_err(PersistenceError::Serialization)
}

pub fn save_reality(reality: &Reality, path: &Path) -> Result<(), PersistenceError> {
    write_bincode(path, reality)
}

pub fn load_reality(path: &Path) -> Result<Reality, PersistenceError> {
    let reality: Reality = read_bincode(path)?;
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
    write_bincode(path, snapshot)
}

pub fn load_snapshot(path: &Path) -> Result<RealitySnapshot, PersistenceError> {
    read_bincode(path)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedReality {
    pub reality: Reality,
    pub public_key: [u8; 32],
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedSnapshot {
    pub snapshot: RealitySnapshot,
    pub public_key: [u8; 32],
    pub signature: Vec<u8>,
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
    let message = bincode::serialize(reality).map_err(PersistenceError::Serialization)?;
    let (public_key, signature) = sign_bytes(&message, signing_key);
    Ok(SignedReality {
        reality: reality.clone(),
        public_key,
        signature: signature.to_vec(),
    })
}

pub fn save_signed_reality(signed: &SignedReality, path: &Path) -> Result<(), PersistenceError> {
    write_bincode(path, signed)
}

pub fn load_signed_reality(
    path: &Path,
    expected_public_key: &[u8; 32],
) -> Result<SignedReality, PersistenceError> {
    let signed: SignedReality = read_bincode(path)?;
    let message = bincode::serialize(&signed.reality).map_err(PersistenceError::Serialization)?;
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
    let message = bincode::serialize(snapshot).map_err(PersistenceError::Serialization)?;
    let (public_key, signature) = sign_bytes(&message, signing_key);
    Ok(SignedSnapshot {
        snapshot: snapshot.clone(),
        public_key,
        signature: signature.to_vec(),
    })
}

pub fn save_signed_snapshot(signed: &SignedSnapshot, path: &Path) -> Result<(), PersistenceError> {
    write_bincode(path, signed)
}

pub fn load_signed_snapshot(
    path: &Path,
    expected_public_key: &[u8; 32],
) -> Result<SignedSnapshot, PersistenceError> {
    let signed: SignedSnapshot = read_bincode(path)?;
    let message = bincode::serialize(&signed.snapshot).map_err(PersistenceError::Serialization)?;
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

    #[test]
    fn reality_roundtrip_preserves_integrity() {
        let mut reality = test_reality();
        let events = vec![
            Event {
                proposed_field: "after".to_string(),
            },
            Event {
                proposed_field: "done".to_string(),
            },
        ];
        let _ = perform_movement_sequence(&mut reality, events).unwrap();

        let path = std::env::temp_dir().join("archimedes-test-reality.bin");
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
        let path = std::env::temp_dir().join("archimedes-test-snapshot.bin");
        save_snapshot(&snapshot, &path).unwrap();
        let loaded = load_snapshot(&path).unwrap();
        assert_eq!(snapshot, loaded);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn tampered_reality_file_is_rejected() {
        let mut reality = test_reality();
        let event = Event {
            proposed_field: "after".to_string(),
        };
        let _ = perform_movement_sequence(&mut reality, vec![event]).unwrap();

        let path = std::env::temp_dir().join("archimedes-test-tampered.bin");
        save_reality(&reality, &path).unwrap();

        let mut bytes = std::fs::read(&path).unwrap();
        let mid = bytes.len() / 2;
        bytes[mid] ^= 0xFF;
        std::fs::write(&path, &bytes).unwrap();

        let result = load_reality(&path);
        assert!(result.is_err());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn signed_reality_roundtrip_works() {
        let mut reality = test_reality();
        let event = Event {
            proposed_field: "after".to_string(),
        };
        let _ = perform_movement_sequence(&mut reality, vec![event]).unwrap();

        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let public_key = signing_key.verifying_key().to_bytes();

        let signed = sign_reality(&reality, &signing_key).unwrap();
        let path = std::env::temp_dir().join("archimedes-test-signed-reality.bin");
        save_signed_reality(&signed, &path).unwrap();

        let loaded = load_signed_reality(&path, &public_key).unwrap();
        assert_eq!(loaded.reality, reality);
        assert_eq!(loaded.public_key, public_key);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn signed_reality_rejects_wrong_key() {
        let mut reality = test_reality();
        let event = Event {
            proposed_field: "after".to_string(),
        };
        let _ = perform_movement_sequence(&mut reality, vec![event]).unwrap();

        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let wrong_key = SigningKey::generate(&mut csprng).verifying_key().to_bytes();

        let signed = sign_reality(&reality, &signing_key).unwrap();
        let path = std::env::temp_dir().join("archimedes-test-signed-wrong.bin");
        save_signed_reality(&signed, &path).unwrap();

        let result = load_signed_reality(&path, &wrong_key);
        assert!(result.is_err());
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
        let path = std::env::temp_dir().join("archimedes-test-signed-snapshot.bin");
        save_signed_snapshot(&signed, &path).unwrap();

        let loaded = load_signed_snapshot(&path, &public_key).unwrap();
        assert_eq!(loaded.snapshot, snapshot);
        let _ = std::fs::remove_file(&path);
    }
}
