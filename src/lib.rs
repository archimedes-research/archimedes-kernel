#![forbid(unsafe_code)]
//! Archimedes minimal kernel v0
//! Proves one minimal Archimedean movement without production concerns.

pub mod movement;
pub mod persistence;
pub mod primitives;
pub mod verification;

pub use movement::{Event, LawCheck, MovementMemory, Transition};
pub use persistence::{
    load_reality, load_signed_reality, load_signed_snapshot, load_snapshot, save_reality,
    save_signed_reality, save_signed_snapshot, save_snapshot, sign_reality, sign_snapshot,
    PersistenceError, SignedReality, SignedSnapshot,
};
pub use primitives::{Boundary, Identity, Law, Reality, State};
pub use verification::{
    detect_drift, plan_sequence, simulate_sequence, Continuity, DriftCheck, Inspection,
    IntegrityReport, MovementError, PlannedSequence, PreflightReport, ProofResult, RealityDiff,
    RealityFingerprint, RealitySnapshot, Replay, SimulationReport, VerificationReport,
};

/// Perform one minimal Archimedean movement.
/// Returns Ok(ProofResult) only if all checks pass.
/// Returns Err(MovementError) on the first failure encountered.
pub fn perform_movement(reality: &mut Reality, event: Event) -> Result<ProofResult, MovementError> {
    // ---- 1. Reality exists ----
    // (reality is passed by reference)

    // ---- 2. Identity confirmed ----
    if reality.identity.0.is_empty() {
        return Err(MovementError::IdentityMissingOrUnstable);
    }

    // ---- 3. Boundary confirmed ----
    if reality.boundary.allowed_values.is_empty() {
        return Err(MovementError::BoundaryMissing);
    }

    // ---- 4. Law confirmed ----
    if reality.law.allowed_transitions.is_empty() {
        return Err(MovementError::LawMissing);
    }

    // ---- 5. State confirmed ----
    if reality.state.field.is_empty() {
        return Err(MovementError::StateMissing);
    }

    // ---- 6. State is inside Boundary ----
    if !reality
        .boundary
        .allowed_values
        .contains(&reality.state.field)
    {
        return Err(MovementError::StateOutsideBoundary);
    }

    // ---- Event received ----
    // event is passed by value.

    // ---- Event must NOT mutate state directly ----
    // By design, event only carries a proposed field; it cannot mutate reality.
    let event_directly_mutated_state = false;

    // ---- 7. LawCheck performed ----
    let law_check_result = reality.law.check(&reality.state, &event);
    let law_check = LawCheck {
        result: law_check_result,
        explanation: if law_check_result {
            "lawful transition".to_string()
        } else {
            "unlawful transition".to_string()
        },
    };
    let law_check_performed = true;

    // ---- 8. Transition occurs before LawCheck? No, we do LawCheck first. ----
    if !law_check.result {
        return Err(MovementError::TransitionNotGroundedInLawCheck);
    }

    // ---- 9. Transition recorded ----
    let after_state = State {
        field: event.proposed_field.clone(),
    };

    // Compute tamper-evident hashes for the movement memory.
    let prev_hash = reality.memory.current_hash();
    let self_hash =
        movement::hash_transition(prev_hash, &reality.state, &after_state, law_check.result);

    let transition = Transition {
        before: reality.state.clone(),
        after: after_state.clone(),
        law_check_valid: law_check.result,
        prev_hash,
        self_hash,
    };

    reality.memory.transitions.push(transition);
    reality.state = after_state;

    // ---- Test hook: simulate hidden mutation (only in test builds) ----
    #[cfg(test)]
    if let Some(hook) = reality.test_hook {
        hook(reality);
    }

    // ---- 10. Inspection available ----
    let inspection = Inspection {
        initial_state: reality.initial_state.clone(),
        memory: reality.memory.clone(),
        current_state: reality.state.clone(),
    };
    let inspection_available = inspection.initial_state == reality.initial_state
        && inspection.memory == reality.memory
        && inspection.current_state == reality.state;

    // ---- 11. Replay ----
    let mut replayed_state = reality.initial_state.clone();
    for t in &reality.memory.transitions {
        if !t.law_check_valid {
            return Err(MovementError::TransitionNotGroundedInLawCheck);
        }
        replayed_state = t.after.clone();
    }
    let replay_result = replayed_state == reality.state;

    // ---- 12. Continuity ----
    let continuity_result = replay_result;

    // ---- 13. DriftCheck ----
    let drift_check = detect_drift(
        &reality.birth_boundary,
        &reality.birth_law,
        &reality.boundary,
        &reality.law,
        replay_result,
    );

    if drift_check.hidden_state_mutation_detected {
        return Err(MovementError::DriftCheckDetectedHiddenStateMutation);
    }
    if drift_check.hidden_boundary_growth_detected {
        return Err(MovementError::DriftCheckDetectedHiddenBoundaryGrowth);
    }
    if drift_check.hidden_law_growth_detected {
        return Err(MovementError::DriftCheckDetectedHiddenLawGrowth);
    }
    if drift_check.permission_drift_detected {
        return Err(MovementError::DriftCheckDetectedPermissionDrift);
    }

    // ---- 14. ProofResult ----
    let proof = ProofResult {
        reality_exists: true,
        identity_confirmed: true,
        active_boundary_confirmed: true,
        active_law_confirmed: true,
        state_confirmed: true,
        event_received: true,
        event_directly_mutated_state,
        law_check_performed,
        law_check_result: law_check.result,
        transition_recorded: true,
        transition_grounded_in_law_check: true,
        movement_memory_recorded: !reality.memory.transitions.is_empty(),
        inspection_available,
        replay_result,
        continuity_result,
        hidden_state_mutation_detected: false,
        hidden_boundary_growth_detected: false,
        hidden_law_growth_detected: false,
        permission_drift_detected: false,
        hidden_drift_required: false,
        proof_status: true,
    };

    // Final sanity check
    if !proof.reality_exists
        || !proof.identity_confirmed
        || !proof.active_boundary_confirmed
        || !proof.active_law_confirmed
        || !proof.state_confirmed
        || !proof.event_received
        || proof.event_directly_mutated_state
        || !proof.law_check_performed
        || !proof.law_check_result
        || !proof.transition_recorded
        || !proof.transition_grounded_in_law_check
        || !proof.movement_memory_recorded
        || !proof.inspection_available
        || !proof.replay_result
        || !proof.continuity_result
        || proof.hidden_state_mutation_detected
        || proof.hidden_boundary_growth_detected
        || proof.hidden_law_growth_detected
        || proof.permission_drift_detected
        || proof.hidden_drift_required
    {
        return Err(MovementError::ProofResultDeclaresPassWithoutEvidence);
    }

    Ok(proof)
}

/// Perform multiple sequential lawful movements.
/// Stops at first error and returns the error.
pub fn perform_movement_sequence(
    reality: &mut Reality,
    events: Vec<Event>,
) -> Result<Vec<ProofResult>, MovementError> {
    let mut proofs = Vec::with_capacity(events.len());
    for event in events {
        let proof = perform_movement(reality, event)?;
        proofs.push(proof);
    }
    Ok(proofs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_reality() -> Reality {
        Reality::new(
            Identity("r0".to_string()),
            Boundary {
                allowed_values: vec!["before".to_string(), "after".to_string()],
            },
            Law {
                allowed_transitions: vec![("before".to_string(), "after".to_string())],
            },
            State {
                field: "before".to_string(),
            },
        )
    }

    fn multi_step_reality() -> Reality {
        Reality::new(
            Identity("r1".to_string()),
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
    fn valid_movement_passes_all_invariants() {
        let mut reality = minimal_reality();
        let event = Event {
            proposed_field: "after".to_string(),
        };

        let proof = perform_movement(&mut reality, event).expect("movement should succeed");

        assert!(proof.proof_status);
        assert!(!proof.event_directly_mutated_state);
        assert!(proof.law_check_result);
        assert!(proof.transition_recorded);
        assert!(proof.continuity_result);
        assert!(!proof.hidden_state_mutation_detected);
        assert!(!proof.hidden_drift_required);
        assert!(reality.memory_integrity());
    }

    #[test]
    fn unlawful_event_fails() {
        let mut reality = minimal_reality();
        let event = Event {
            proposed_field: "before".to_string(),
        };

        let result = perform_movement(&mut reality, event);
        assert!(result.is_err());
        assert!(reality.memory().transitions.is_empty());
    }

    #[test]
    fn missing_identity_fails() {
        let mut reality = minimal_reality();
        reality.identity.0 = String::new();
        let event = Event {
            proposed_field: "after".to_string(),
        };
        let result = perform_movement(&mut reality, event);
        assert_eq!(
            result.unwrap_err(),
            MovementError::IdentityMissingOrUnstable
        );
    }

    #[test]
    fn state_outside_boundary_fails() {
        let mut reality = minimal_reality();
        reality.state.field = "outside".to_string();
        let event = Event {
            proposed_field: "after".to_string(),
        };
        let result = perform_movement(&mut reality, event);
        assert_eq!(result.unwrap_err(), MovementError::StateOutsideBoundary);
    }

    #[test]
    fn hidden_boundary_growth_detected_via_test_hook() {
        let mut reality = minimal_reality();
        reality.test_hook = Some(|r: &mut Reality| {
            r.boundary.allowed_values.push("sneaky".to_string());
        });
        let event = Event {
            proposed_field: "after".to_string(),
        };

        let result = perform_movement(&mut reality, event);
        assert_eq!(
            result.unwrap_err(),
            MovementError::DriftCheckDetectedHiddenBoundaryGrowth
        );
        assert_eq!(reality.memory().transitions.len(), 1);
    }

    #[test]
    fn valid_sequence_moves_through_all_steps() {
        let mut reality = multi_step_reality();
        let events = vec![
            Event {
                proposed_field: "after".to_string(),
            },
            Event {
                proposed_field: "done".to_string(),
            },
        ];

        let proofs =
            perform_movement_sequence(&mut reality, events).expect("sequence should succeed");

        assert_eq!(proofs.len(), 2);
        assert!(proofs.iter().all(|p| p.proof_status));
        assert_eq!(reality.state.field, "done");
        assert_eq!(reality.memory().transitions.len(), 2);
        assert!(reality.memory_integrity());
    }

    #[test]
    fn sequence_stops_on_unlawful_event() {
        let mut reality = multi_step_reality();
        let events = vec![
            Event {
                proposed_field: "after".to_string(),
            },
            Event {
                proposed_field: "after".to_string(),
            },
        ];

        let result = perform_movement_sequence(&mut reality, events);

        assert!(result.is_err());
        assert_eq!(reality.state.field, "after");
        assert_eq!(reality.memory().transitions.len(), 1);
    }

    #[test]
    fn inspection_method_returns_proof_path() {
        let mut reality = minimal_reality();
        let event = Event {
            proposed_field: "after".to_string(),
        };
        let _ = perform_movement(&mut reality, event).unwrap();

        let inspection = reality.inspect();
        assert_eq!(inspection.initial_state, reality.initial_state);
        assert_eq!(inspection.memory, reality.memory);
        assert_eq!(inspection.current_state, reality.state);
    }

    #[test]
    fn replay_method_matches_current_state() {
        let mut reality = minimal_reality();
        let event = Event {
            proposed_field: "after".to_string(),
        };
        let _ = perform_movement(&mut reality, event).unwrap();

        let replay = reality.replay();
        assert!(replay.passed);
        assert_eq!(replay.replayed_state, reality.state);
    }

    #[test]
    fn continuity_method_reflects_state_match() {
        let mut reality = minimal_reality();
        let event = Event {
            proposed_field: "after".to_string(),
        };
        let _ = perform_movement(&mut reality, event).unwrap();

        let continuity = reality.continuity();
        assert!(continuity.preserved);
    }

    #[test]
    fn continuity_detects_state_mismatch() {
        let mut reality = minimal_reality();
        let event = Event {
            proposed_field: "after".to_string(),
        };
        let _ = perform_movement(&mut reality, event).unwrap();

        // Simulate hidden mutation by directly changing state (only possible in test code)
        reality.state.field = "sneaky".to_string();

        let replay = reality.replay();
        assert!(!replay.passed);
        let continuity = reality.continuity();
        assert!(!continuity.preserved);
    }

    #[test]
    fn verify_returns_bundled_report() {
        let mut reality = minimal_reality();
        let event = Event {
            proposed_field: "after".to_string(),
        };
        let _ = perform_movement(&mut reality, event).unwrap();

        let report = reality.verify();
        assert!(report.replay.passed);
        assert!(report.continuity.preserved);
        assert!(report.memory_integrity);
        assert_eq!(report.inspection.current_state, reality.state);
    }

    #[test]
    fn memory_integrity_preserved_after_valid_movement() {
        let mut reality = minimal_reality();
        let event = Event {
            proposed_field: "after".to_string(),
        };
        let _ = perform_movement(&mut reality, event).unwrap();
        assert!(reality.memory_integrity());
    }

    #[test]
    fn memory_tampering_detected() {
        let mut reality = minimal_reality();
        let event = Event {
            proposed_field: "after".to_string(),
        };
        let _ = perform_movement(&mut reality, event).unwrap();

        // Tamper with a past transition's recorded after state
        reality.memory.transitions[0].after.field = "sneaky".to_string();

        assert!(!reality.memory_integrity());
        let replay = reality.replay();
        assert!(!replay.passed);
    }

    #[test]
    fn drift_between_movements_detected() {
        let mut reality = multi_step_reality();
        let first_event = Event {
            proposed_field: "after".to_string(),
        };
        let _ = perform_movement(&mut reality, first_event).unwrap();

        // Simulate hidden mutation between movements by directly changing boundary
        reality.boundary.allowed_values.push("sneaky".to_string());

        let second_event = Event {
            proposed_field: "done".to_string(),
        };
        let result = perform_movement(&mut reality, second_event);
        assert_eq!(
            result.unwrap_err(),
            MovementError::DriftCheckDetectedHiddenBoundaryGrowth
        );
    }

    #[test]
    fn public_drift_check_reports_clean_when_no_drift() {
        let mut reality = minimal_reality();
        let event = Event {
            proposed_field: "after".to_string(),
        };
        let _ = perform_movement(&mut reality, event).unwrap();

        let drift = reality.drift_check();
        assert!(!drift.hidden_state_mutation_detected);
        assert!(!drift.hidden_boundary_growth_detected);
        assert!(!drift.hidden_law_growth_detected);
        assert!(!drift.permission_drift_detected);
        assert!(!drift.hidden_drift_required);
    }

    #[test]
    fn public_drift_check_detects_hidden_boundary_growth() {
        let mut reality = minimal_reality();
        let event = Event {
            proposed_field: "after".to_string(),
        };
        let _ = perform_movement(&mut reality, event).unwrap();

        // Mutate boundary after movement
        reality.boundary.allowed_values.push("sneaky".to_string());

        let drift = reality.drift_check();
        assert!(drift.hidden_boundary_growth_detected);
        assert!(drift.hidden_drift_required);
    }

    #[test]
    fn compose_returns_valid_view() {
        let mut reality = multi_step_reality();
        let events = vec![
            Event {
                proposed_field: "after".to_string(),
            },
            Event {
                proposed_field: "done".to_string(),
            },
        ];
        let _ = perform_movement_sequence(&mut reality, events).unwrap();

        let comp = reality.memory().compose(0, 1).expect("valid composition");
        assert_eq!(comp.start_state.field, "before");
        assert_eq!(comp.end_state.field, "done");
        assert_eq!(comp.hash, reality.memory().current_hash());
        assert!(comp.verify(reality.memory()));
    }

    #[test]
    fn compose_rejects_invalid_range() {
        let mut reality = multi_step_reality();
        let events = vec![
            Event {
                proposed_field: "after".to_string(),
            },
            Event {
                proposed_field: "done".to_string(),
            },
        ];
        let _ = perform_movement_sequence(&mut reality, events).unwrap();

        assert!(reality.memory().compose(1, 0).is_none());
        assert!(reality.memory().compose(0, 2).is_none()); // end out of bounds
    }

    #[test]
    fn composition_verifies_correctly_and_detects_tamper() {
        let mut reality = multi_step_reality();
        let events = vec![
            Event {
                proposed_field: "after".to_string(),
            },
            Event {
                proposed_field: "done".to_string(),
            },
        ];
        let _ = perform_movement_sequence(&mut reality, events).unwrap();

        let comp = reality.memory().compose(0, 1).unwrap();
        assert!(comp.verify(reality.memory()));

        // Tamper with a transition inside the composition segment
        reality.memory.transitions[1].after.field = "sneaky".to_string();
        assert!(!comp.verify(reality.memory()));
    }

    #[test]
    fn plan_sequence_succeeds_without_mutating_original() {
        let reality = multi_step_reality();
        let events = vec![
            Event {
                proposed_field: "after".to_string(),
            },
            Event {
                proposed_field: "done".to_string(),
            },
        ];

        let planned = plan_sequence(&reality, events).expect("plan should succeed");

        assert_eq!(planned.final_state.field, "done");
        assert_eq!(planned.transition_count, 2);
        assert_eq!(planned.proofs.len(), 2);
        // Original unchanged
        assert_eq!(reality.state.field, "before");
        assert_eq!(reality.memory().transitions.len(), 0);
    }

    #[test]
    fn plan_sequence_fails_on_unlawful_event() {
        let reality = multi_step_reality();
        let events = vec![
            Event {
                proposed_field: "after".to_string(),
            },
            Event {
                proposed_field: "after".to_string(),
            },
        ];

        let result = plan_sequence(&reality, events);
        assert!(result.is_err());
        // Original unchanged
        assert_eq!(reality.state.field, "before");
        assert_eq!(reality.memory().transitions.len(), 0);
    }

    #[test]
    fn planned_sequence_matches_executed_sequence() {
        let mut reality = multi_step_reality();
        let events = vec![
            Event {
                proposed_field: "after".to_string(),
            },
            Event {
                proposed_field: "done".to_string(),
            },
        ];

        let planned = plan_sequence(&reality, events.clone()).expect("plan should succeed");
        let actual =
            perform_movement_sequence(&mut reality, events).expect("execute should succeed");

        assert_eq!(planned.final_state, reality.state);
        assert_eq!(planned.transition_count, reality.memory().transitions.len());
        assert_eq!(planned.proofs, actual);
    }

    #[test]
    fn identical_realities_have_identical_fingerprints() {
        let reality1 = multi_step_reality();
        let reality2 = multi_step_reality();

        assert_eq!(reality1.fingerprint(), reality2.fingerprint());
    }

    #[test]
    fn mutation_changes_fingerprint() {
        let mut reality = multi_step_reality();
        let original_fingerprint = reality.fingerprint();

        let event = Event {
            proposed_field: "after".to_string(),
        };
        let _ = perform_movement(&mut reality, event).unwrap();

        let new_fingerprint = reality.fingerprint();
        assert_ne!(original_fingerprint, new_fingerprint);
    }

    #[test]
    fn integrity_report_reflects_clean_state() {
        let mut reality = multi_step_reality();
        let events = vec![
            Event {
                proposed_field: "after".to_string(),
            },
            Event {
                proposed_field: "done".to_string(),
            },
        ];
        let _ = perform_movement_sequence(&mut reality, events).unwrap();

        let report = reality.integrity_report();

        assert_eq!(report.fingerprint, reality.fingerprint());
        assert!(!report.drift.hidden_drift_required);
        assert!(report.memory_integrity);
        assert!(report.replay.passed);
        assert!(report.continuity.preserved);
        assert_eq!(report.state, reality.state);
        assert_eq!(report.transition_count, 2);
    }

    #[test]
    fn integrity_report_detects_drift() {
        let mut reality = multi_step_reality();
        let events = vec![
            Event {
                proposed_field: "after".to_string(),
            },
            Event {
                proposed_field: "done".to_string(),
            },
        ];
        let _ = perform_movement_sequence(&mut reality, events).unwrap();

        // Simulate hidden boundary growth after movement
        reality.boundary.allowed_values.push("sneaky".to_string());

        let report = reality.integrity_report();

        assert!(report.drift.hidden_boundary_growth_detected);
        assert!(report.drift.hidden_drift_required);
        // Memory integrity still true because boundary drift does not affect hash chain
        assert!(report.memory_integrity);
    }

    #[test]
    fn would_accept_lawful_event() {
        let reality = multi_step_reality();
        let event = Event {
            proposed_field: "after".to_string(),
        };
        assert!(reality.would_accept(&event));
    }

    #[test]
    fn would_accept_rejects_unlawful_event() {
        let reality = multi_step_reality();
        let event = Event {
            proposed_field: "before".to_string(),
        };
        assert!(!reality.would_accept(&event));
    }

    #[test]
    fn would_accept_rejects_empty_event() {
        let reality = multi_step_reality();
        let event = Event {
            proposed_field: String::new(),
        };
        assert!(!reality.would_accept(&event));
    }

    #[test]
    fn simulate_sequence_succeeds_and_does_not_mutate_original() {
        let reality = multi_step_reality();
        let events = vec![
            Event {
                proposed_field: "after".to_string(),
            },
            Event {
                proposed_field: "done".to_string(),
            },
        ];

        let report = simulate_sequence(&reality, events).expect("simulation should succeed");

        assert_eq!(report.planned.final_state.field, "done");
        assert_eq!(report.planned.transition_count, 2);
        assert!(report.integrity.memory_integrity);
        assert!(report.integrity.replay.passed);
        assert!(report.integrity.continuity.preserved);
        assert!(!report.integrity.drift.hidden_drift_required);
        assert!(report.composition.is_some());
        assert_eq!(report.fingerprint, report.integrity.fingerprint);

        // Original unchanged
        assert_eq!(reality.state.field, "before");
        assert_eq!(reality.memory().transitions.len(), 0);
    }

    #[test]
    fn simulate_sequence_fails_on_unlawful_event() {
        let reality = multi_step_reality();
        let events = vec![
            Event {
                proposed_field: "after".to_string(),
            },
            Event {
                proposed_field: "after".to_string(),
            },
        ];

        let result = simulate_sequence(&reality, events);
        assert!(result.is_err());
        assert_eq!(reality.state.field, "before");
        assert_eq!(reality.memory().transitions.len(), 0);
    }

    #[test]
    fn simulated_report_matches_actual_execution() {
        let mut reality = multi_step_reality();
        let events = vec![
            Event {
                proposed_field: "after".to_string(),
            },
            Event {
                proposed_field: "done".to_string(),
            },
        ];

        let simulation =
            simulate_sequence(&reality, events.clone()).expect("simulation should succeed");
        let actual =
            perform_movement_sequence(&mut reality, events).expect("execution should succeed");

        assert_eq!(simulation.planned.proofs, actual);
        assert_eq!(simulation.planned.final_state, reality.state);
        assert_eq!(
            simulation.planned.transition_count,
            reality.memory().transitions.len()
        );
        assert_eq!(simulation.fingerprint, reality.fingerprint());
        assert_eq!(simulation.integrity, reality.integrity_report());
    }

    #[test]
    fn identical_realities_have_all_same_diff() {
        let r1 = multi_step_reality();
        let r2 = multi_step_reality();

        let diff = r1.diff(&r2);

        assert!(diff.identity_same);
        assert!(diff.boundary_same);
        assert!(diff.law_same);
        assert!(diff.state_same);
        assert!(diff.initial_state_same);
        assert!(diff.birth_boundary_same);
        assert!(diff.birth_law_same);
        assert!(diff.memory_hash_same);
        assert!(diff.transition_count_same);
        assert!(diff.fingerprint_same);
    }

    #[test]
    fn diff_detects_state_change() {
        let mut r1 = multi_step_reality();
        let r2 = multi_step_reality();

        let event = Event {
            proposed_field: "after".to_string(),
        };
        let _ = perform_movement(&mut r1, event).unwrap();

        let diff = r1.diff(&r2);

        assert!(!diff.state_same);
        assert!(!diff.memory_hash_same);
        assert!(!diff.transition_count_same);
        assert!(!diff.fingerprint_same);
        assert!(diff.boundary_same);
        assert!(diff.law_same);
    }

    #[test]
    fn diff_detects_boundary_change() {
        let mut r1 = multi_step_reality();
        let r2 = multi_step_reality();

        // Directly mutate boundary inside test code
        r1.boundary.allowed_values.push("sneaky".to_string());

        let diff = r1.diff(&r2);

        assert!(!diff.boundary_same);
        assert!(diff.fingerprint_same);
        assert!(diff.identity_same);
        assert!(diff.law_same);
    }

    #[test]
    fn preflight_sequence_all_lawful() {
        let reality = multi_step_reality();
        let events = vec![
            Event {
                proposed_field: "after".to_string(),
            },
            Event {
                proposed_field: "done".to_string(),
            },
        ];

        let report = reality.preflight_sequence(&events);

        assert_eq!(report.results, vec![true, true]);
        assert!(report.sequence_lawful);
        assert_eq!(
            report.final_state.as_ref().map(|s| s.field.as_str()),
            Some("done")
        );
        assert_eq!(report.transition_count, 2);
    }

    #[test]
    fn preflight_sequence_stops_at_first_unlawful() {
        let reality = multi_step_reality();
        let events = vec![
            Event {
                proposed_field: "after".to_string(),
            },
            Event {
                proposed_field: "after".to_string(),
            },
        ];

        let report = reality.preflight_sequence(&events);

        assert_eq!(report.results, vec![true, false]);
        assert!(!report.sequence_lawful);
        assert!(report.final_state.is_none());
        assert_eq!(report.transition_count, 1);
    }

    #[test]
    fn preflight_does_not_mutate_original() {
        let reality = multi_step_reality();
        let events = vec![
            Event {
                proposed_field: "after".to_string(),
            },
            Event {
                proposed_field: "done".to_string(),
            },
        ];

        let _ = reality.preflight_sequence(&events);

        assert_eq!(reality.state.field, "before");
        assert_eq!(reality.memory().transitions.len(), 0);
    }

    #[test]
    fn snapshot_matches_current_reality() {
        let mut reality = multi_step_reality();
        let event = Event {
            proposed_field: "after".to_string(),
        };
        let _ = perform_movement(&mut reality, event).unwrap();

        let snapshot = reality.snapshot();

        assert_eq!(snapshot.identity, reality.identity);
        assert_eq!(snapshot.boundary, reality.boundary);
        assert_eq!(snapshot.law, reality.law);
        assert_eq!(snapshot.state, reality.state);
        assert_eq!(snapshot.initial_state, reality.initial_state);
        assert_eq!(snapshot.birth_boundary, reality.birth_boundary);
        assert_eq!(snapshot.birth_law, reality.birth_law);
        assert_eq!(snapshot.memory_hash, reality.memory.current_hash());
        assert_eq!(snapshot.transition_count, reality.memory.transitions.len());
        assert_eq!(snapshot.fingerprint, reality.fingerprint());
        assert_eq!(snapshot.integrity, reality.integrity_report());
    }

    #[test]
    fn snapshot_changes_after_movement() {
        let mut reality = multi_step_reality();
        let before = reality.snapshot();

        let event = Event {
            proposed_field: "after".to_string(),
        };
        let _ = perform_movement(&mut reality, event).unwrap();

        let after = reality.snapshot();

        assert_ne!(before, after);
    }

    #[test]
    fn snapshot_is_readonly_by_clone_equality() {
        let reality = multi_step_reality();
        let snapshot1 = reality.snapshot();
        let snapshot2 = reality.snapshot();

        assert_eq!(snapshot1, snapshot2);
    }

    #[test]
    fn snapshot_matches_current_when_no_change() {
        let mut reality = multi_step_reality();
        let event = Event {
            proposed_field: "after".to_string(),
        };
        let _ = perform_movement(&mut reality, event).unwrap();

        let snapshot = reality.snapshot();
        assert!(snapshot.matches_current(&reality));
    }

    #[test]
    fn snapshot_does_not_match_after_further_movement() {
        let mut reality = multi_step_reality();
        let event = Event {
            proposed_field: "after".to_string(),
        };
        let _ = perform_movement(&mut reality, event).unwrap();

        let snapshot = reality.snapshot();

        let second_event = Event {
            proposed_field: "done".to_string(),
        };
        let _ = perform_movement(&mut reality, second_event).unwrap();

        assert!(!snapshot.matches_current(&reality));
    }

    #[test]
    fn snapshot_diff_against_current_detects_changes() {
        let mut reality = multi_step_reality();
        let event = Event {
            proposed_field: "after".to_string(),
        };
        let _ = perform_movement(&mut reality, event).unwrap();

        let snapshot = reality.snapshot();

        let second_event = Event {
            proposed_field: "done".to_string(),
        };
        let _ = perform_movement(&mut reality, second_event).unwrap();

        let diff = snapshot.diff_against(&reality);
        assert!(!diff.state_same);
        assert!(!diff.memory_hash_same);
        assert!(!diff.transition_count_same);
        assert!(!diff.fingerprint_same);
    }
}
