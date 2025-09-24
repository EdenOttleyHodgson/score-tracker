use std::{
    collections::{HashMap, HashSet},
    iter::zip,
};

use serde::{Deserialize, Serialize};

use crate::state::{ID, error::WagerMutationError};

#[derive(Serialize, Clone, PartialEq, Eq, Debug, schemars::JsonSchema)]
pub struct Wager {
    id: ID,
    participant_bets: HashMap<ID, i64>,
    //Keys are the choices, values is who chose
    participant_choices: HashMap<ID, HashSet<ID>>,
    outcomes: HashMap<ID, WagerOutcome>,
    name: String,
}
impl Wager {
    pub fn new(id: ID, name: String, outcomes: Vec<WagerOutcome>) -> Self {
        Self {
            id,
            name,
            outcomes: outcomes
                .into_iter()
                .map(|outcome| (outcome.id, outcome))
                .collect(),
            participant_bets: HashMap::new(),
            participant_choices: HashMap::new(),
        }
    }
    pub fn join(&mut self, user: ID, outcome: ID, amount: i64) -> Result<(), WagerMutationError> {
        if self.participant_bets.contains_key(&user) {
            Err(WagerMutationError::UserAlreadyExists {
                user_id: user,
                wager_id: self.id,
            })
        } else {
            if let Some(choice) = self.participant_choices.get_mut(&outcome) {
                if !choice.insert(user) {
                    //This like might be unreachable?
                    return Err(WagerMutationError::UserAlreadyChose {
                        user_id: user,
                        outcome_id: outcome,
                    });
                }
            } else {
                self.participant_choices
                    .insert(outcome, HashSet::from([user]));
            }
            self.participant_bets.insert(user, amount.abs());
            Ok(())
        }
    }
    pub fn remove_user(&mut self, user: ID) {
        self.participant_bets.remove(&user);
        for choice_set in self.participant_choices.values_mut() {
            choice_set.remove(&user);
        }
    }
    pub fn resolve(&mut self, outcome: ID) -> Result<Vec<WagerResult>, WagerMutationError> {
        let score_mult: f128 = (self
            .outcomes
            .get(&outcome)
            .ok_or(WagerMutationError::NonexistentOutcome {
                outcome_id: outcome,
                wager_id: self.id,
            })?
            .odds) as f128
            / 100.0;
        let winner_ids = if let Some(ids) = self.participant_choices.get(&outcome) {
            ids
        } else {
            log::warn!("No winners in bet!");
            return Ok(Vec::new());
        };
        Ok(self
            .participant_bets
            .iter()
            .filter_map(|(id, bet)| {
                if winner_ids.contains(id) {
                    let score_diff: i64 = (((*bet as f128) * score_mult).round() as i64) + bet;
                    Some(WagerResult {
                        participant: *id,
                        score_diff,
                    })
                } else {
                    None
                }
            })
            .collect())
    }
}
#[cfg(test)]
impl Wager {
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn outcomes(&self) -> Vec<WagerOutcome> {
        use itertools::Itertools;

        self.outcomes
            .clone()
            .into_iter()
            .sorted_by_key(|x| x.0)
            .map(|x| x.1)
            .collect()
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct WagerResult {
    pub participant: ID,
    pub score_diff: i64,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, schemars::JsonSchema)]
pub struct WagerOutcome {
    name: String,
    id: ID,
    description: String,
    odds: usize,
}

#[cfg(test)]
impl WagerOutcome {
    pub fn new(name: String, description: String, odds: usize, id: ID) -> Self {
        Self {
            name,
            description,
            id,
            odds,
        }
    }
}
