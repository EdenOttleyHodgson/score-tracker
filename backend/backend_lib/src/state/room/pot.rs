use std::collections::HashSet;

use serde::Serialize;

use crate::state::{ID, error::PotMutationError};

#[derive(Serialize, Clone, PartialEq, Eq, Debug, schemars::JsonSchema)]
pub struct Pot {
    pot_id: ID,
    total_score: i64,
    score_requirement: i64,
    participants: HashSet<ID>,
    description: String,
}

impl Pot {
    pub fn new(id: ID, score_requirement: i64, desc: String) -> Pot {
        Pot {
            pot_id: id,
            total_score: 0,
            score_requirement,
            participants: HashSet::new(),
            description: desc,
        }
    }
    pub fn join(&mut self, id: ID) -> Result<(), PotMutationError> {
        if self.participants.insert(id) {
            self.total_score += self.score_requirement;
            Ok(())
        } else {
            Err(PotMutationError::UserAlreadyExists {
                user_id: id,
                pot_id: self.pot_id,
            })
        }
    }
    pub fn remove_user(&mut self, id: ID) -> Result<(), PotMutationError> {
        if self.participants.remove(&id) {
            Ok(())
        } else {
            Err(PotMutationError::UserNotInPot {
                user_id: id,
                pot_id: self.pot_id,
            })
        }
    }
    pub fn score_req(&self) -> i64 {
        self.score_requirement
    }
    pub fn resolve(&self) -> i64 {
        self.total_score
    }
    pub fn participants(&self) -> impl Iterator<Item = &ID> {
        self.participants.iter()
    }
}
