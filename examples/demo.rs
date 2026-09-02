use std::env;

use archimedes_kernel::{
    load_reality, load_signed_reality, load_snapshot,
    movement::Event,
    perform_movement_sequence, plan_sequence,
    primitives::{Boundary, Identity, Law, Reality, State},
    save_reality, save_signed_reality, save_snapshot, sign_reality, simulate_sequence,
    SignedReality,
};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

fn main() {
    let mut reality = Reality::new(
        Identity("demo-reality".to_string()),
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
    .expect("valid Reality construction");

    let original = reality.clone();

    let events = vec![
        Event {
            proposed_field: "after".to_string(),
        },
        Event {
            proposed_field: "done".to_string(),
        },
    ];

    println!("Preflight Checks:");
    for (i, event) in events.iter().enumerate() {
        println!(
            "  event {} ({:?}) would_accept = {}",
            i + 1,
            event.proposed_field,
            reality.would_accept(event)
        );
    }

    let preflight = reality.preflight_sequence(&events);
    println!("\nPreflight Sequence Report:");
    println!("  results = {:?}", preflight.results);
    println!("  sequence lawful = {}", preflight.sequence_lawful);
    println!("  transition count = {}", preflight.transition_count);
    match &preflight.final_state {
        Some(state) => println!("  final state = {:?}", state),
        None => println!("  final state = None"),
    }

    match plan_sequence(&reality, events.clone()) {
        Ok(planned) => {
            println!("\nPlanned Sequence:");
            println!("  final state = {:?}", planned.final_state);
            println!("  transitions planned = {}", planned.transition_count);
        }
        Err(e) => eprintln!("Planned sequence failed: {:?}", e),
    }

    match simulate_sequence(&reality, events.clone()) {
        Ok(simulation) => {
            println!("\nSimulation Report:");
            println!(
                "  planned final state = {:?}",
                simulation.planned.final_state
            );
            println!(
                "  planned transition count = {}",
                simulation.planned.transition_count
            );
            println!("  simulated fingerprint = {:?}", simulation.fingerprint);
            println!(
                "  simulated memory integrity = {}",
                simulation.integrity.memory_integrity
            );
            println!(
                "  simulated drift required = {}",
                simulation.integrity.drift.hidden_drift_required
            );
            println!(
                "  simulated composition present = {}",
                simulation.composition.is_some()
            );
        }
        Err(e) => eprintln!("Simulation failed: {:?}", e),
    }

    match perform_movement_sequence(&mut reality, events) {
        Ok(proofs) => {
            println!("\nMovement sequence succeeded.");
            for (i, proof) in proofs.iter().enumerate() {
                println!("Step {}: proof_status = {}", i + 1, proof.proof_status);
                println!("  law_check_result = {}", proof.law_check_result);
                println!("  continuity_result = {}", proof.continuity_result);
                println!("  hidden_drift_required = {}", proof.hidden_drift_required);
            }

            let report = reality.verify();
            println!("\nVerification Report:");
            println!("  replay passed = {}", report.replay.passed);
            println!("  continuity preserved = {}", report.continuity.preserved);
            println!("  memory integrity = {}", report.memory_integrity);
            println!("  final state = {:?}", report.inspection.current_state);

            let drift = reality.drift_check();
            println!("\nDrift Check:");
            println!("  hidden drift required = {}", drift.hidden_drift_required);
            println!(
                "  hidden boundary growth = {}",
                drift.hidden_boundary_growth_detected
            );
            println!("  hidden law growth = {}", drift.hidden_law_growth_detected);

            let comp = reality
                .memory()
                .compose(0, 1)
                .expect("composition should exist");
            println!("\nMovement Composition:");
            println!("  start state: {:?}", comp.start_state);
            println!("  end state: {:?}", comp.end_state);
            println!("  verified: {}", comp.verify(reality.memory()));

            let fingerprint = reality.fingerprint();
            println!("\nReality Fingerprint:");
            println!("  fingerprint = {:?}", fingerprint);

            let integrity = reality.integrity_report();
            println!("\nUnified Integrity Report:");
            println!("  fingerprint = {:?}", integrity.fingerprint);
            println!("  memory integrity = {}", integrity.memory_integrity);
            println!(
                "  drift required = {}",
                integrity.drift.hidden_drift_required
            );
            println!("  replay passed = {}", integrity.replay.passed);
            println!(
                "  continuity preserved = {}",
                integrity.continuity.preserved
            );
            println!("  state = {:?}", integrity.state);
            println!("  transition count = {}", integrity.transition_count);

            let diff = reality.diff(&original);
            println!("\nReality Diff (original -> current):");
            println!("  identity same = {}", diff.identity_same);
            println!("  boundary same = {}", diff.boundary_same);
            println!("  law same = {}", diff.law_same);
            println!("  state same = {}", diff.state_same);
            println!("  initial state same = {}", diff.initial_state_same);
            println!("  birth boundary same = {}", diff.birth_boundary_same);
            println!("  birth law same = {}", diff.birth_law_same);
            println!("  memory hash same = {}", diff.memory_hash_same);
            println!("  transition count same = {}", diff.transition_count_same);
            println!("  fingerprint same = {}", diff.fingerprint_same);

            let snapshot = reality.snapshot();
            println!("\nReality Snapshot:");
            println!("  identity = {:?}", snapshot.identity);
            println!("  state = {:?}", snapshot.state);
            println!("  transition count = {}", snapshot.transition_count);
            println!("  memory hash = {:?}", snapshot.memory_hash);
            println!("  fingerprint = {:?}", snapshot.fingerprint);

            let snapshot_matches = snapshot.matches_current(&reality);
            println!("\nSnapshot Comparison:");
            println!("  matches current = {}", snapshot_matches);

            let snapshot_diff = snapshot.diff_against(&reality);
            println!("  snapshot diff vs current:");
            println!("    identity same = {}", snapshot_diff.identity_same);
            println!("    boundary same = {}", snapshot_diff.boundary_same);
            println!("    law same = {}", snapshot_diff.law_same);
            println!("    state same = {}", snapshot_diff.state_same);
            println!(
                "    initial state same = {}",
                snapshot_diff.initial_state_same
            );
            println!("    memory hash same = {}", snapshot_diff.memory_hash_same);
            println!(
                "    transition count same = {}",
                snapshot_diff.transition_count_same
            );
            println!("    fingerprint same = {}", snapshot_diff.fingerprint_same);

            // Persistence demonstration
            let tmp = env::temp_dir();
            let reality_path = tmp.join("archimedes-demo-reality.bin");
            let snapshot_path = tmp.join("archimedes-demo-snapshot.bin");

            save_reality(&reality, &reality_path).unwrap();
            save_snapshot(&snapshot, &snapshot_path).unwrap();

            let loaded_reality = load_reality(&reality_path).unwrap();
            let loaded_snapshot = load_snapshot(&snapshot_path).unwrap();

            println!("\nUnsigned Persistence:");
            println!(
                "  loaded reality integrity = {}",
                loaded_reality.memory_integrity()
            );
            println!(
                "  loaded reality matches current = {}",
                loaded_reality.diff(&reality).fingerprint_same
            );
            println!(
                "  loaded snapshot matches saved = {}",
                loaded_snapshot.matches_current(&reality)
            );

            // Signed persistence demonstration
            let mut csprng = OsRng;
            let signing_key = SigningKey::generate(&mut csprng);
            let public_key = signing_key.verifying_key().to_bytes();

            let signed_reality = sign_reality(&reality, &signing_key).unwrap();
            let signed_path = tmp.join("archimedes-demo-signed-reality.bin");
            save_signed_reality(&signed_reality, &signed_path).unwrap();

            let loaded_signed: SignedReality =
                load_signed_reality(&signed_path, &public_key).unwrap();
            println!("\nSigned Persistence:");
            println!(
                "  signed reality integrity = {}",
                loaded_signed.reality.memory_integrity()
            );
            println!(
                "  signed reality matches current = {}",
                loaded_signed.reality.diff(&reality).fingerprint_same
            );
            println!("  signature valid = true");

            let _ = std::fs::remove_file(&reality_path);
            let _ = std::fs::remove_file(&snapshot_path);
            let _ = std::fs::remove_file(&signed_path);

            println!("\nFinal state: {:?}", reality.state());
            println!(
                "Transitions recorded: {}",
                reality.memory().transitions.len()
            );
        }
        Err(e) => {
            eprintln!("Movement sequence failed: {:?}", e);
        }
    }
}
