use serde::{Deserialize, Serialize};

use crate::movement::{Event, MovementMemory};

pub(crate) fn is_valid_state_value(value: &str) -> bool {
    !value.is_empty()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Boundary {
    pub allowed_values: Vec<String>,
}

impl Boundary {
    pub(crate) fn has_valid_state_domain(&self) -> bool {
        !self.allowed_values.is_empty()
            && self
                .allowed_values
                .iter()
                .all(|value| is_valid_state_value(value))
    }

    pub(crate) fn contains_state_value(&self, value: &str) -> bool {
        is_valid_state_value(value) && self.allowed_values.iter().any(|allowed| allowed == value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Law {
    pub allowed_transitions: Vec<(String, String)>,
}

impl Law {
    pub(crate) fn has_valid_state_domain(&self) -> bool {
        !self.allowed_transitions.is_empty()
            && self
                .allowed_transitions
                .iter()
                .all(|(from, to)| is_valid_state_value(from) && is_valid_state_value(to))
    }

    pub fn check(&self, current_state: &State, event: &Event) -> bool {
        if !current_state.is_valid() || !is_valid_state_value(&event.proposed_field) {
            return false;
        }

        self.allowed_transitions.iter().any(|(from, to)| {
            is_valid_state_value(from)
                && is_valid_state_value(to)
                && from == &current_state.field
                && to == &event.proposed_field
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub field: String,
}

impl State {
    pub(crate) fn is_valid(&self) -> bool {
        is_valid_state_value(&self.field)
    }
}

#[derive(Debug, Clone, Serialize)]
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

#[derive(Serialize, Deserialize)]
pub(crate) struct RealityWire {
    identity: Identity,
    boundary: Boundary,
    law: Law,
    state: State,
    initial_state: State,
    memory: MovementMemory,
    birth_boundary: Boundary,
    birth_law: Law,
}

impl RealityWire {
    pub(crate) fn into_validated(self) -> Result<Reality, crate::verification::MovementError> {
        let reality = Reality {
            identity: self.identity,
            boundary: self.boundary,
            law: self.law,
            state: self.state,
            initial_state: self.initial_state,
            memory: self.memory,
            birth_boundary: self.birth_boundary,
            birth_law: self.birth_law,
            #[cfg(test)]
            test_hook: None,
        };

        reality.validate()?;

        Ok(reality)
    }
}

impl<'de> Deserialize<'de> for Reality {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = RealityWire::deserialize(deserializer)?;

        wire.into_validated().map_err(serde::de::Error::custom)
    }
}

impl Reality {
    pub fn new(
        identity: Identity,
        boundary: Boundary,
        law: Law,
        state: State,
    ) -> Result<Self, crate::verification::MovementError> {
        let reality = Self {
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
        };

        reality.validate()?;

        Ok(reality)
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
