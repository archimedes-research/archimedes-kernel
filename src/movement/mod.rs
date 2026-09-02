use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::primitives::State;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HashValue(pub [u8; 32]);

impl HashValue {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

fn write_bytes(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

fn write_str(hasher: &mut Sha256, s: &str) {
    write_bytes(hasher, s.as_bytes());
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub proposed_field: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LawCheck {
    pub result: bool,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transition {
    pub before: State,
    pub after: State,
    pub law_check_valid: bool,
    pub prev_hash: HashValue,
    pub self_hash: HashValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MovementMemory {
    pub transitions: Vec<Transition>,
}

pub fn hash_transition(
    prev_hash: HashValue,
    before: &State,
    after: &State,
    law_check_valid: bool,
) -> HashValue {
    let mut hasher = Sha256::new();
    hasher.update(prev_hash.0);
    write_str(&mut hasher, &before.field);
    write_str(&mut hasher, &after.field);
    hasher.update([law_check_valid as u8]);

    let result = hasher.finalize();
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&result);
    HashValue(bytes)
}

impl MovementMemory {
    pub fn current_hash(&self) -> HashValue {
        self.transitions
            .last()
            .map(|t| t.self_hash)
            .unwrap_or(HashValue([0u8; 32]))
    }

    pub fn verify_integrity(&self) -> bool {
        let mut prev_hash = HashValue([0u8; 32]);
        for t in &self.transitions {
            let computed = hash_transition(prev_hash, &t.before, &t.after, t.law_check_valid);
            if computed != t.self_hash || t.prev_hash != prev_hash {
                return false;
            }
            prev_hash = t.self_hash;
        }
        true
    }

    pub fn verify_semantic_continuity(&self, initial_state: &State) -> bool {
        let Some(first) = self.transitions.first() else {
            return true;
        };

        if &first.before != initial_state {
            return false;
        }

        self.transitions
            .windows(2)
            .all(|pair| pair[0].after == pair[1].before)
    }

    pub fn compose(&self, start: usize, end: usize) -> Option<MovementComposition> {
        if start > end || end >= self.transitions.len() {
            return None;
        }

        let first = &self.transitions[start];
        let last = &self.transitions[end];

        Some(MovementComposition {
            start_index: start,
            end_index: end,
            start_state: first.before.clone(),
            end_state: last.after.clone(),
            hash: last.self_hash,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MovementComposition {
    pub start_index: usize,
    pub end_index: usize,
    pub start_state: State,
    pub end_state: State,
    pub hash: HashValue,
}

impl MovementComposition {
    pub fn verify(&self, memory: &MovementMemory) -> bool {
        if self.start_index > self.end_index || self.end_index >= memory.transitions.len() {
            return false;
        }

        let first = &memory.transitions[self.start_index];
        let last = &memory.transitions[self.end_index];

        if first.before != self.start_state || last.after != self.end_state {
            return false;
        }

        if last.self_hash != self.hash {
            return false;
        }

        if self.start_index > 0 && memory.transitions[self.start_index - 1].after != first.before {
            return false;
        }

        if !memory.transitions[self.start_index..=self.end_index]
            .windows(2)
            .all(|pair| pair[0].after == pair[1].before)
        {
            return false;
        }

        let mut prev_hash = if self.start_index == 0 {
            HashValue([0u8; 32])
        } else {
            memory.transitions[self.start_index - 1].self_hash
        };

        for t in &memory.transitions[self.start_index..=self.end_index] {
            if t.prev_hash != prev_hash {
                return false;
            }
            let computed = hash_transition(prev_hash, &t.before, &t.after, t.law_check_valid);
            if computed != t.self_hash {
                return false;
            }
            prev_hash = t.self_hash;
        }

        true
    }
}
