use std::collections::HashSet;

use serde::Serialize;

use crate::state::{ID, error::RoomMutationError};

#[derive(Serialize, Clone, PartialEq, Eq, Debug, schemars::JsonSchema)]
pub struct MemberState {
    id: ID,
    name: String,
    score: i64,
    current_wagers: HashSet<ID>,
    current_pots: HashSet<ID>,
}

impl MemberState {
    pub fn new(name: String, id: ID) -> Self {
        Self {
            name,
            score: 0,
            id,
            current_wagers: HashSet::new(),
            current_pots: HashSet::new(),
        }
    }

    pub fn score(&self) -> i64 {
        self.score
    }

    pub fn set_score(&mut self, score: i64) -> Result<(), RoomMutationError> {
        if score < 0 {
            return Err(RoomMutationError::NegativeScore);
        }
        self.score = score;
        Ok(())
    }

    pub fn id(&self) -> ID {
        self.id
    }

    pub fn current_wagers_mut(&mut self) -> &mut HashSet<ID> {
        &mut self.current_wagers
    }

    pub fn current_pots_mut(&mut self) -> &mut HashSet<ID> {
        &mut self.current_pots
    }
}
