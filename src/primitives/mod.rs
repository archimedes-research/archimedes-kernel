use serde::{Deserialize, Serialize};

use crate::movement::{Event, MovementMemory};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Boundary {
    pub allowed_values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Law {
    pub allowed_transitions: Vec<(String, String)>,
}

impl Law {
    pub fn check(&self, current_state: &State, event: &Event) -> bool {
        self.allowed_transitions
            .iter()
            .any(|(from, to)| from == &current_state.field && to == &event.proposed_field)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reality {
    pub(crate) identity: Identity,
    pub(crate) boundary: Boundary,
    pub(crate) law: Law,
    pub(crate) state: State,
    pub(crate) initial_state: State,
    pub(crate) memory: MovementMemory,
    pub(crate) birth_boundary: Boundary,
    pub(crate) birth_law: Law,
    #[cfg(test)]
    #[serde(skip)]
    pub(crate) test_hook: Option<fn(&mut Reality)>,
}

impl Reality {
    pub fn new(identity: Identity, boundary: Boundary, law: Law, state: State) -> Self {
        Self {
            identity,
            boundary: boundary.clone(),
            law: law.clone(),
            initial_state: state.clone(),
            state,
            memory: MovementMemory::default(),
            birth_boundary: boundary,
            birth_law: law,
            #[cfg(test)]
            test_hook: None,
        }
    }

    pub fn identity(&self) -> &Identity {
        &self.identity
    }
    pub fn boundary(&self) -> &Boundary {
        &self.boundary
    }
    pub fn law(&self) -> &Law {
        &self.law
    }
    pub fn state(&self) -> &State {
        &self.state
    }
    pub fn initial_state(&self) -> &State {
        &self.initial_state
    }
    pub fn memory(&self) -> &MovementMemory {
        &self.memory
    }
}

impl PartialEq for Reality {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
            && self.boundary == other.boundary
            && self.law == other.law
            && self.state == other.state
            && self.initial_state == other.initial_state
            && self.memory == other.memory
            && self.birth_boundary == other.birth_boundary
            && self.birth_law == other.birth_law
        // test_hook intentionally ignored
    }
}

impl Eq for Reality {}
