use archimedes_kernel::{
    movement::{hash_transition, HashValue, MovementMemory, Transition},
    perform_movement, Boundary, Event, Identity, Law, MovementError, Reality, State,
};

use serde::Serialize;

fn s(value: &str) -> String {
    value.to_string()
}

fn ordinary_reality() -> Reality {
    Reality::new(
        Identity(s("ordinary")),
        Boundary {
            allowed_values: vec![s("a"), s("b"), s("c")],
        },
        Law {
            allowed_transitions: vec![(s("a"), s("b")), (s("b"), s("c"))],
        },
        State { field: s("a") },
    )
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

fn decode_wire(wire: RealityWire) -> Reality {
    let bytes = postcard::to_allocvec(&wire).expect("wire serialization must succeed");

    postcard::from_bytes::<Reality>(&bytes).expect(
        "ENG-02 still permits raw Reality Deserialize; \
             ENG-03 will close this ingestion boundary",
    )
}

#[test]
fn boundary_escape_is_rejected_without_mutation() {
    let mut reality = Reality::new(
        Identity(s("boundary")),
        Boundary {
            allowed_values: vec![s("inside")],
        },
        Law {
            allowed_transitions: vec![(s("inside"), s("outside"))],
        },
        State { field: s("inside") },
    );

    let before = reality.clone();

    let result = perform_movement(
        &mut reality,
        Event {
            proposed_field: s("outside"),
        },
    );

    assert_eq!(
        result,
        Err(MovementError::LawReferencesStateOutsideBoundary),
    );

    assert_eq!(reality, before);
}

#[test]
fn preflight_matches_execution_for_invalid_identity() {
    let reality = Reality::new(
        Identity(String::new()),
        Boundary {
            allowed_values: vec![s("a"), s("b")],
        },
        Law {
            allowed_transitions: vec![(s("a"), s("b"))],
        },
        State { field: s("a") },
    );

    let event = Event {
        proposed_field: s("b"),
    };

    let preflight = reality.preflight_sequence(std::slice::from_ref(&event));

    let mut actual = reality.clone();

    let execution = perform_movement(&mut actual, event);

    assert!(!preflight.sequence_lawful);
    assert_eq!(preflight.results, vec![false]);
    assert_eq!(preflight.transition_count, 0);
    assert_eq!(preflight.final_state, None);

    assert_eq!(execution, Err(MovementError::IdentityMissingOrUnstable),);

    assert_eq!(actual, reality);
}

#[test]
fn hash_valid_but_discontinuous_memory_is_semantically_invalid() {
    let zero = HashValue([0u8; 32]);

    let a = State { field: s("a") };
    let b = State { field: s("b") };
    let x = State { field: s("x") };
    let c = State { field: s("c") };

    let h1 = hash_transition(zero, &a, &b, true);

    let first = Transition {
        before: a.clone(),
        after: b,
        law_check_valid: true,
        prev_hash: zero,
        self_hash: h1,
    };

    let h2 = hash_transition(h1, &x, &c, true);

    let second = Transition {
        before: x,
        after: c,
        law_check_valid: true,
        prev_hash: h1,
        self_hash: h2,
    };

    let memory = MovementMemory {
        transitions: vec![first, second],
    };

    assert!(memory.verify_integrity());

    assert!(!memory.verify_semantic_continuity(&a));

    let composition = memory.compose(0, 1).expect("composition exists");

    assert!(!composition.verify(&memory));
}

#[test]
fn failed_movement_preserves_state_memory_and_hash() {
    let law = Law {
        allowed_transitions: vec![(s("a"), s("b"))],
    };

    let boundary = Boundary {
        allowed_values: vec![s("a"), s("b"), s("hidden")],
    };

    let mut reality = decode_wire(RealityWire {
        identity: Identity(s("atomicity")),
        boundary,
        law: law.clone(),
        state: State { field: s("a") },
        initial_state: State { field: s("a") },
        memory: MovementMemory::default(),
        birth_boundary: Boundary {
            allowed_values: vec![s("a"), s("b")],
        },
        birth_law: law,
    });

    let before = reality.clone();
    let hash_before = reality.memory().current_hash();

    let result = perform_movement(
        &mut reality,
        Event {
            proposed_field: s("b"),
        },
    );

    assert_eq!(
        result,
        Err(MovementError::DriftCheckDetectedHiddenBoundaryGrowth),
    );

    assert_eq!(reality, before);

    assert_eq!(reality.memory().current_hash(), hash_before,);
}

#[test]
fn discontinuous_reality_fails_replay_continuity_and_validation() {
    let zero = HashValue([0u8; 32]);

    let a = State { field: s("a") };
    let b = State { field: s("b") };
    let x = State { field: s("x") };
    let c = State { field: s("c") };

    let h1 = hash_transition(zero, &a, &b, true);

    let h2 = hash_transition(h1, &x, &c, true);

    let memory = MovementMemory {
        transitions: vec![
            Transition {
                before: a.clone(),
                after: b,
                law_check_valid: true,
                prev_hash: zero,
                self_hash: h1,
            },
            Transition {
                before: x,
                after: c.clone(),
                law_check_valid: true,
                prev_hash: h1,
                self_hash: h2,
            },
        ],
    };

    let boundary = Boundary {
        allowed_values: vec![s("a"), s("b"), s("x"), s("c")],
    };

    let law = Law {
        allowed_transitions: vec![(s("a"), s("b")), (s("x"), s("c"))],
    };

    let reality = decode_wire(RealityWire {
        identity: Identity(s("discontinuous")),
        boundary: boundary.clone(),
        law: law.clone(),
        state: c,
        initial_state: a,
        memory,
        birth_boundary: boundary,
        birth_law: law,
    });

    assert!(reality.memory_integrity());
    assert!(!reality.replay().passed);
    assert!(!reality.continuity().preserved);

    assert_eq!(
        reality.validate(),
        Err(MovementError::MovementHistoryDiscontinuous),
    );
}

#[test]
fn invalid_deserialized_reality_is_detected_by_semantic_contract() {
    let reality = decode_wire(RealityWire {
        identity: Identity(String::new()),
        boundary: Boundary {
            allowed_values: vec![s("inside")],
        },
        law: Law {
            allowed_transitions: vec![(s("inside"), s("outside"))],
        },
        state: State {
            field: s("outside"),
        },
        initial_state: State {
            field: s("outside"),
        },
        memory: MovementMemory::default(),
        birth_boundary: Boundary {
            allowed_values: vec![s("inside")],
        },
        birth_law: Law {
            allowed_transitions: vec![(s("inside"), s("outside"))],
        },
    });

    assert_eq!(
        reality.validate(),
        Err(MovementError::IdentityMissingOrUnstable),
    );
}

#[test]
fn ordinary_lawful_movement_remains_valid() {
    let mut reality = ordinary_reality();

    let proof = perform_movement(
        &mut reality,
        Event {
            proposed_field: s("b"),
        },
    )
    .expect("lawful movement must succeed");

    assert!(proof.proof_status);
    assert_eq!(reality.state().field, "b");
    assert!(reality.memory_integrity());
    assert!(reality.replay().passed);
    assert!(reality.continuity().preserved);
    assert_eq!(reality.validate(), Ok(()));
}
