use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::movement::{Event, HashValue, MovementComposition, MovementMemory};
use crate::primitives::{Boundary, Identity, Law, Reality, State};

fn write_bytes(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

fn write_str(hasher: &mut Sha256, s: &str) {
    write_bytes(hasher, s.as_bytes());
}

fn write_string_vec(hasher: &mut Sha256, items: &[String]) {
    hasher.update((items.len() as u64).to_le_bytes());
    for item in items {
        write_str(hasher, item);
    }
}

fn write_transition_vec(hasher: &mut Sha256, items: &[(String, String)]) {
    hasher.update((items.len() as u64).to_le_bytes());
    for (from, to) in items {
        write_str(hasher, from);
        write_str(hasher, to);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inspection {
    pub initial_state: State,
    pub memory: MovementMemory,
    pub current_state: State,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Replay {
    pub replayed_state: State,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Continuity {
    pub preserved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriftCheck {
    pub hidden_state_mutation_detected: bool,
    pub hidden_boundary_growth_detected: bool,
    pub hidden_law_growth_detected: bool,
    pub permission_drift_detected: bool,
    pub hidden_drift_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofResult {
    pub reality_exists: bool,
    pub identity_confirmed: bool,
    pub active_boundary_confirmed: bool,
    pub active_law_confirmed: bool,
    pub state_confirmed: bool,
    pub event_received: bool,
    pub event_directly_mutated_state: bool,
    pub law_check_performed: bool,
    pub law_check_result: bool,
    pub transition_recorded: bool,
    pub transition_grounded_in_law_check: bool,
    pub movement_memory_recorded: bool,
    pub inspection_available: bool,
    pub replay_result: bool,
    pub continuity_result: bool,
    pub hidden_state_mutation_detected: bool,
    pub hidden_boundary_growth_detected: bool,
    pub hidden_law_growth_detected: bool,
    pub permission_drift_detected: bool,
    pub hidden_drift_required: bool,
    pub proof_status: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementError {
    RealityMissing,
    IdentityMissingOrUnstable,
    BoundaryMissing,
    LawMissing,
    StateMissing,
    StateOutsideBoundary,
    EventMutatesStateDirectly,
    LawCheckMissing,
    TransitionBeforeLawCheck,
    TransitionUnrecorded,
    TransitionNotGroundedInLawCheck,
    MovementMemoryMissing,
    InspectionHidesProofPath,
    ReplayChecksOnlyFinalState,
    ContinuityAssertedWithoutReplay,
    DriftCheckDetectedHiddenStateMutation,
    DriftCheckDetectedHiddenBoundaryGrowth,
    DriftCheckDetectedHiddenLawGrowth,
    DriftCheckDetectedPermissionDrift,
    ProofResultDeclaresPassWithoutEvidence,
    InitialStateOutsideBoundary,
    LawReferencesStateOutsideBoundary,
    MovementMemoryIntegrityInvalid,
    MovementHistoryDiscontinuous,
    MovementHistoryNotGroundedInLaw,
}

impl std::fmt::Display for MovementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for MovementError {}

pub fn detect_drift(
    original_boundary: &Boundary,
    original_law: &Law,
    current_boundary: &Boundary,
    current_law: &Law,
    replay_result: bool,
) -> DriftCheck {
    let hidden_state_mutation_detected = !replay_result;
    let hidden_boundary_growth_detected = original_boundary != current_boundary;
    let hidden_law_growth_detected = original_law != current_law;
    let permission_drift_detected = hidden_law_growth_detected;
    let hidden_drift_required = hidden_state_mutation_detected
        || hidden_boundary_growth_detected
        || hidden_law_growth_detected
        || permission_drift_detected;

    DriftCheck {
        hidden_state_mutation_detected,
        hidden_boundary_growth_detected,
        hidden_law_growth_detected,
        permission_drift_detected,
        hidden_drift_required,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationReport {
    pub inspection: Inspection,
    pub replay: Replay,
    pub continuity: Continuity,
    pub memory_integrity: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedSequence {
    pub final_state: State,
    pub transition_count: usize,
    pub proofs: Vec<ProofResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealityFingerprint(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub fingerprint: RealityFingerprint,
    pub drift: DriftCheck,
    pub memory_integrity: bool,
    pub replay: Replay,
    pub continuity: Continuity,
    pub state: State,
    pub transition_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimulationReport {
    pub planned: PlannedSequence,
    pub integrity: IntegrityReport,
    pub composition: Option<MovementComposition>,
    pub fingerprint: RealityFingerprint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealityDiff {
    pub identity_same: bool,
    pub boundary_same: bool,
    pub law_same: bool,
    pub state_same: bool,
    pub initial_state_same: bool,
    pub birth_boundary_same: bool,
    pub birth_law_same: bool,
    pub memory_hash_same: bool,
    pub transition_count_same: bool,
    pub fingerprint_same: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightReport {
    pub results: Vec<bool>,
    pub sequence_lawful: bool,
    pub final_state: Option<State>,
    pub transition_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotValidationError {
    IdentityMissing,
    BoundaryMissing,
    LawMissing,
    StateMissing,
    StateOutsideBoundary,
    InitialStateOutsideBoundary,
    LawReferencesStateOutsideBoundary,
    FingerprintMismatch,
    IntegrityFingerprintMismatch,
    IntegrityStateMismatch,
    IntegrityTransitionCountMismatch,
    MemoryIntegrityNotEstablished,
    ReplayStateMismatch,
    ContinuityMismatch,
    DriftMismatch,
    HiddenDriftReported,
    EmptyHistoryMismatch,
}

impl std::fmt::Display for SnapshotValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SnapshotValidationError {}

fn fingerprint_components(
    identity: &Identity,
    birth_boundary: &Boundary,
    birth_law: &Law,
    state: &State,
    initial_state: &State,
    memory_hash: HashValue,
) -> RealityFingerprint {
    let mut hasher = Sha256::new();

    write_str(&mut hasher, &identity.0);
    write_string_vec(&mut hasher, &birth_boundary.allowed_values);
    write_transition_vec(&mut hasher, &birth_law.allowed_transitions);
    write_str(&mut hasher, &state.field);
    write_str(&mut hasher, &initial_state.field);
    hasher.update(memory_hash.0);

    let result = hasher.finalize();
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&result);

    RealityFingerprint(bytes)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealitySnapshot {
    pub identity: Identity,
    pub boundary: Boundary,
    pub law: Law,
    pub state: State,
    pub initial_state: State,
    pub birth_boundary: Boundary,
    pub birth_law: Law,
    pub memory_hash: HashValue,
    pub transition_count: usize,
    pub fingerprint: RealityFingerprint,
    pub integrity: IntegrityReport,
}

impl RealitySnapshot {
    pub fn validate(&self) -> Result<(), SnapshotValidationError> {
        if self.identity.0.is_empty() {
            return Err(SnapshotValidationError::IdentityMissing);
        }

        if self.boundary.allowed_values.is_empty() || self.birth_boundary.allowed_values.is_empty()
        {
            return Err(SnapshotValidationError::BoundaryMissing);
        }

        if self.law.allowed_transitions.is_empty() || self.birth_law.allowed_transitions.is_empty()
        {
            return Err(SnapshotValidationError::LawMissing);
        }

        if self.state.field.is_empty() || self.initial_state.field.is_empty() {
            return Err(SnapshotValidationError::StateMissing);
        }

        if !self.boundary.allowed_values.contains(&self.state.field) {
            return Err(SnapshotValidationError::StateOutsideBoundary);
        }

        if !self
            .birth_boundary
            .allowed_values
            .contains(&self.initial_state.field)
        {
            return Err(SnapshotValidationError::InitialStateOutsideBoundary);
        }

        let active_law_inside_boundary = self.law.allowed_transitions.iter().all(|(from, to)| {
            self.boundary.allowed_values.contains(from) && self.boundary.allowed_values.contains(to)
        });

        let birth_law_inside_boundary =
            self.birth_law.allowed_transitions.iter().all(|(from, to)| {
                self.birth_boundary.allowed_values.contains(from)
                    && self.birth_boundary.allowed_values.contains(to)
            });

        if !active_law_inside_boundary || !birth_law_inside_boundary {
            return Err(SnapshotValidationError::LawReferencesStateOutsideBoundary);
        }

        let expected_fingerprint = fingerprint_components(
            &self.identity,
            &self.birth_boundary,
            &self.birth_law,
            &self.state,
            &self.initial_state,
            self.memory_hash,
        );

        if self.fingerprint != expected_fingerprint {
            return Err(SnapshotValidationError::FingerprintMismatch);
        }

        if self.integrity.fingerprint != self.fingerprint {
            return Err(SnapshotValidationError::IntegrityFingerprintMismatch);
        }

        if self.integrity.state != self.state {
            return Err(SnapshotValidationError::IntegrityStateMismatch);
        }

        if self.integrity.transition_count != self.transition_count {
            return Err(SnapshotValidationError::IntegrityTransitionCountMismatch);
        }

        if !self.integrity.memory_integrity {
            return Err(SnapshotValidationError::MemoryIntegrityNotEstablished);
        }

        if self.integrity.continuity.preserved != self.integrity.replay.passed {
            return Err(SnapshotValidationError::ContinuityMismatch);
        }

        if self.integrity.replay.passed && self.integrity.replay.replayed_state != self.state {
            return Err(SnapshotValidationError::ReplayStateMismatch);
        }

        let expected_drift = detect_drift(
            &self.birth_boundary,
            &self.birth_law,
            &self.boundary,
            &self.law,
            self.integrity.replay.passed,
        );

        if self.integrity.drift != expected_drift {
            return Err(SnapshotValidationError::DriftMismatch);
        }

        if self.integrity.drift.hidden_drift_required {
            return Err(SnapshotValidationError::HiddenDriftReported);
        }

        if self.transition_count == 0
            && (self.memory_hash != HashValue([0u8; 32]) || self.state != self.initial_state)
        {
            return Err(SnapshotValidationError::EmptyHistoryMismatch);
        }

        Ok(())
    }

    pub fn matches_current(&self, reality: &Reality) -> bool {
        if self.validate().is_err() {
            return false;
        }

        let diff = self.diff_against(reality);
        diff.identity_same
            && diff.boundary_same
            && diff.law_same
            && diff.state_same
            && diff.initial_state_same
            && diff.birth_boundary_same
            && diff.birth_law_same
            && diff.memory_hash_same
            && diff.transition_count_same
            && diff.fingerprint_same
    }

    pub fn diff_against(&self, reality: &Reality) -> RealityDiff {
        RealityDiff {
            identity_same: self.identity == reality.identity,
            boundary_same: self.boundary == reality.boundary,
            law_same: self.law == reality.law,
            state_same: self.state == reality.state,
            initial_state_same: self.initial_state == reality.initial_state,
            birth_boundary_same: self.birth_boundary == reality.birth_boundary,
            birth_law_same: self.birth_law == reality.birth_law,
            memory_hash_same: self.memory_hash == reality.memory.current_hash(),
            transition_count_same: self.transition_count == reality.memory.transitions.len(),
            fingerprint_same: self.fingerprint == reality.fingerprint(),
        }
    }
}

impl Reality {
    pub fn validate(&self) -> Result<(), MovementError> {
        if self.identity.0.is_empty() {
            return Err(MovementError::IdentityMissingOrUnstable);
        }

        if self.boundary.allowed_values.is_empty() || self.birth_boundary.allowed_values.is_empty()
        {
            return Err(MovementError::BoundaryMissing);
        }

        if self.law.allowed_transitions.is_empty() || self.birth_law.allowed_transitions.is_empty()
        {
            return Err(MovementError::LawMissing);
        }

        if self.state.field.is_empty() || self.initial_state.field.is_empty() {
            return Err(MovementError::StateMissing);
        }

        if !self.boundary.allowed_values.contains(&self.state.field) {
            return Err(MovementError::StateOutsideBoundary);
        }

        if !self
            .birth_boundary
            .allowed_values
            .contains(&self.initial_state.field)
        {
            return Err(MovementError::InitialStateOutsideBoundary);
        }

        let active_law_inside_boundary = self.law.allowed_transitions.iter().all(|(from, to)| {
            self.boundary.allowed_values.contains(from) && self.boundary.allowed_values.contains(to)
        });

        let birth_law_inside_boundary =
            self.birth_law.allowed_transitions.iter().all(|(from, to)| {
                self.birth_boundary.allowed_values.contains(from)
                    && self.birth_boundary.allowed_values.contains(to)
            });

        if !active_law_inside_boundary || !birth_law_inside_boundary {
            return Err(MovementError::LawReferencesStateOutsideBoundary);
        }

        if !self.memory.verify_integrity() {
            return Err(MovementError::MovementMemoryIntegrityInvalid);
        }

        if !self.memory.verify_semantic_continuity(&self.initial_state) {
            return Err(MovementError::MovementHistoryDiscontinuous);
        }

        for transition in &self.memory.transitions {
            if !transition.law_check_valid {
                return Err(MovementError::MovementHistoryNotGroundedInLaw);
            }

            let event = Event {
                proposed_field: transition.after.field.clone(),
            };

            if !self.birth_law.check(&transition.before, &event) {
                return Err(MovementError::MovementHistoryNotGroundedInLaw);
            }
        }

        let replay = self.replay();

        if !replay.passed {
            return Err(MovementError::DriftCheckDetectedHiddenStateMutation);
        }

        let drift = detect_drift(
            &self.birth_boundary,
            &self.birth_law,
            &self.boundary,
            &self.law,
            replay.passed,
        );

        if drift.hidden_state_mutation_detected {
            return Err(MovementError::DriftCheckDetectedHiddenStateMutation);
        }

        if drift.hidden_boundary_growth_detected {
            return Err(MovementError::DriftCheckDetectedHiddenBoundaryGrowth);
        }

        if drift.hidden_law_growth_detected {
            return Err(MovementError::DriftCheckDetectedHiddenLawGrowth);
        }

        if drift.permission_drift_detected {
            return Err(MovementError::DriftCheckDetectedPermissionDrift);
        }

        Ok(())
    }

    pub fn inspect(&self) -> Inspection {
        Inspection {
            initial_state: self.initial_state.clone(),
            memory: self.memory.clone(),
            current_state: self.state.clone(),
        }
    }
    pub fn replay(&self) -> Replay {
        if !self.memory.verify_integrity()
            || !self.memory.verify_semantic_continuity(&self.initial_state)
        {
            return Replay {
                replayed_state: self.initial_state.clone(),
                passed: false,
            };
        }

        let mut replayed_state = self.initial_state.clone();

        for transition in &self.memory.transitions {
            if !transition.law_check_valid || transition.before != replayed_state {
                return Replay {
                    replayed_state,
                    passed: false,
                };
            }

            let event = Event {
                proposed_field: transition.after.field.clone(),
            };

            if !self.birth_law.check(&transition.before, &event) {
                return Replay {
                    replayed_state,
                    passed: false,
                };
            }

            replayed_state = transition.after.clone();
        }

        let passed = replayed_state == self.state;

        Replay {
            replayed_state,
            passed,
        }
    }
    pub fn continuity(&self) -> Continuity {
        Continuity {
            preserved: self.replay().passed,
        }
    }
    pub fn memory_integrity(&self) -> bool {
        self.memory.verify_integrity()
    }
    pub fn verify(&self) -> VerificationReport {
        VerificationReport {
            inspection: self.inspect(),
            replay: self.replay(),
            continuity: self.continuity(),
            memory_integrity: self.memory_integrity(),
        }
    }
    pub fn drift_check(&self) -> DriftCheck {
        detect_drift(
            &self.birth_boundary,
            &self.birth_law,
            &self.boundary,
            &self.law,
            self.replay().passed,
        )
    }
    pub fn fingerprint(&self) -> RealityFingerprint {
        fingerprint_components(
            &self.identity,
            &self.birth_boundary,
            &self.birth_law,
            &self.state,
            &self.initial_state,
            self.memory.current_hash(),
        )
    }
    pub fn integrity_report(&self) -> IntegrityReport {
        IntegrityReport {
            fingerprint: self.fingerprint(),
            drift: self.drift_check(),
            memory_integrity: self.memory_integrity(),
            replay: self.replay(),
            continuity: self.continuity(),
            state: self.state().clone(),
            transition_count: self.memory().transitions.len(),
        }
    }
    pub fn would_accept(&self, event: &Event) -> bool {
        let mut candidate = self.clone();
        crate::perform_movement(&mut candidate, event.clone()).is_ok()
    }
    pub fn diff(&self, other: &Reality) -> RealityDiff {
        RealityDiff {
            identity_same: self.identity == other.identity,
            boundary_same: self.boundary == other.boundary,
            law_same: self.law == other.law,
            state_same: self.state == other.state,
            initial_state_same: self.initial_state == other.initial_state,
            birth_boundary_same: self.birth_boundary == other.birth_boundary,
            birth_law_same: self.birth_law == other.birth_law,
            memory_hash_same: self.memory.current_hash() == other.memory.current_hash(),
            transition_count_same: self.memory.transitions.len() == other.memory.transitions.len(),
            fingerprint_same: self.fingerprint() == other.fingerprint(),
        }
    }
    pub fn preflight_sequence(&self, events: &[Event]) -> PreflightReport {
        let mut clone = self.clone();
        let mut results = Vec::with_capacity(events.len());
        let mut accepted = 0usize;

        for event in events {
            match crate::perform_movement(&mut clone, event.clone()) {
                Ok(_) => {
                    results.push(true);
                    accepted += 1;
                }
                Err(_) => {
                    results.push(false);
                    return PreflightReport {
                        results,
                        sequence_lawful: false,
                        final_state: None,
                        transition_count: accepted,
                    };
                }
            }
        }

        PreflightReport {
            results,
            sequence_lawful: true,
            final_state: Some(clone.state().clone()),
            transition_count: accepted,
        }
    }
    pub fn snapshot(&self) -> RealitySnapshot {
        let snapshot = RealitySnapshot {
            identity: self.identity.clone(),
            boundary: self.boundary.clone(),
            law: self.law.clone(),
            state: self.state.clone(),
            initial_state: self.initial_state.clone(),
            birth_boundary: self.birth_boundary.clone(),
            birth_law: self.birth_law.clone(),
            memory_hash: self.memory.current_hash(),
            transition_count: self.memory.transitions.len(),
            fingerprint: self.fingerprint(),
            integrity: self.integrity_report(),
        };

        debug_assert!(snapshot.validate().is_ok());

        snapshot
    }
}

pub fn plan_sequence(
    reality: &Reality,
    events: Vec<Event>,
) -> Result<PlannedSequence, MovementError> {
    let mut clone = reality.clone();
    let proofs = crate::perform_movement_sequence(&mut clone, events)?;
    Ok(PlannedSequence {
        final_state: clone.state().clone(),
        transition_count: clone.memory().transitions.len(),
        proofs,
    })
}

pub fn simulate_sequence(
    reality: &Reality,
    events: Vec<Event>,
) -> Result<SimulationReport, MovementError> {
    let mut clone = reality.clone();
    let proofs = crate::perform_movement_sequence(&mut clone, events)?;

    let planned = PlannedSequence {
        final_state: clone.state().clone(),
        transition_count: clone.memory().transitions.len(),
        proofs,
    };
    let integrity = clone.integrity_report();
    let composition = if !clone.memory().transitions.is_empty() {
        clone
            .memory()
            .compose(0, clone.memory().transitions.len() - 1)
    } else {
        None
    };
    let fingerprint = clone.fingerprint();

    Ok(SimulationReport {
        planned,
        integrity,
        composition,
        fingerprint,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_drift_no_change() {
        let boundary = Boundary {
            allowed_values: vec!["a".into(), "b".into()],
        };
        let law = Law {
            allowed_transitions: vec![("a".to_string(), "b".to_string())],
        };
        let drift = detect_drift(&boundary, &law, &boundary, &law, true);
        assert!(!drift.hidden_drift_required);
    }

    #[test]
    fn detect_drift_boundary_change() {
        let b1 = Boundary {
            allowed_values: vec!["a".into(), "b".into()],
        };
        let b2 = Boundary {
            allowed_values: vec!["a".into(), "b".into(), "c".into()],
        };
        let law = Law {
            allowed_transitions: vec![("a".to_string(), "b".to_string())],
        };
        let drift = detect_drift(&b1, &law, &b2, &law, true);
        assert!(drift.hidden_boundary_growth_detected);
        assert!(drift.hidden_drift_required);
    }

    #[test]
    fn detect_drift_law_change() {
        let boundary = Boundary {
            allowed_values: vec!["a".into(), "b".into()],
        };
        let law1 = Law {
            allowed_transitions: vec![("a".to_string(), "b".to_string())],
        };
        let law2 = Law {
            allowed_transitions: vec![
                ("a".to_string(), "b".to_string()),
                ("b".to_string(), "c".to_string()),
            ],
        };
        let drift = detect_drift(&boundary, &law1, &boundary, &law2, true);
        assert!(drift.hidden_law_growth_detected);
        assert!(drift.permission_drift_detected);
    }

    #[test]
    fn detect_drift_state_mutation() {
        let boundary = Boundary {
            allowed_values: vec!["a".into(), "b".into()],
        };
        let law = Law {
            allowed_transitions: vec![("a".to_string(), "b".to_string())],
        };
        let drift = detect_drift(&boundary, &law, &boundary, &law, false);
        assert!(drift.hidden_state_mutation_detected);
    }
}
