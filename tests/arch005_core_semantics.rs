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
    .expect("ordinary Reality must be valid")
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

#[test]
fn constructor_rejects_law_that_escapes_boundary() {
    let result = Reality::new(
        Identity(s("boundary")),
        Boundary {
            allowed_values: vec![s("inside")],
        },
        Law {
            allowed_transitions: vec![(s("inside"), s("outside"))],
        },
        State { field: s("inside") },
    );

    assert_eq!(
        result,
        Err(MovementError::LawReferencesStateOutsideBoundary),
    );
}

#[test]
fn preflight_matches_execution_for_unlawful_event() {
    let reality = ordinary_reality();

    let event = Event {
        proposed_field: s("c"),
    };

    let preflight = reality.preflight_sequence(std::slice::from_ref(&event));

    let mut actual = reality.clone();

    let execution = perform_movement(&mut actual, event);

    assert!(!preflight.sequence_lawful);
    assert_eq!(preflight.results, vec![false]);
    assert_eq!(preflight.transition_count, 0);
    assert_eq!(preflight.final_state, None);

    assert_eq!(
        execution,
        Err(MovementError::TransitionNotGroundedInLawCheck),
    );

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
fn failed_public_movement_preserves_state_memory_and_hash() {
    let mut reality = ordinary_reality();

    let before = reality.clone();
    let hash_before = reality.memory().current_hash();

    let result = perform_movement(
        &mut reality,
        Event {
            proposed_field: s("c"),
        },
    );

    assert_eq!(result, Err(MovementError::TransitionNotGroundedInLawCheck),);

    assert_eq!(reality, before);

    assert_eq!(reality.memory().current_hash(), hash_before,);
}

#[test]
fn discontinuous_reality_deserialization_is_rejected() {
    let zero = HashValue([0u8; 32]);

    let a = State { field: s("a") };
    let b = State { field: s("b") };
    let x = State { field: s("x") };
    let c = State { field: s("c") };

    let h1 = hash_transition(zero, &a, &b, true);

    let h2 = hash_transition(h1, &x, &c, true);

    let boundary = Boundary {
        allowed_values: vec![s("a"), s("b"), s("x"), s("c")],
    };

    let law = Law {
        allowed_transitions: vec![(s("a"), s("b")), (s("x"), s("c"))],
    };

    let wire = RealityWire {
        identity: Identity(s("discontinuous")),
        boundary: boundary.clone(),
        law: law.clone(),
        state: c.clone(),
        initial_state: a.clone(),
        memory: MovementMemory {
            transitions: vec![
                Transition {
                    before: a,
                    after: b,
                    law_check_valid: true,
                    prev_hash: zero,
                    self_hash: h1,
                },
                Transition {
                    before: x,
                    after: c,
                    law_check_valid: true,
                    prev_hash: h1,
                    self_hash: h2,
                },
            ],
        },
        birth_boundary: boundary,
        birth_law: law,
    };

    let bytes = postcard::to_allocvec(&wire).expect("wire serialization");

    assert!(postcard::from_bytes::<Reality>(&bytes,).is_err());
}

#[test]
fn invalid_reality_deserialization_is_rejected() {
    let wire = RealityWire {
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
    };

    let bytes = postcard::to_allocvec(&wire).expect("wire serialization");

    assert!(postcard::from_bytes::<Reality>(&bytes,).is_err());
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
