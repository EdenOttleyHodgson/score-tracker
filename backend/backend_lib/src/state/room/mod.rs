use std::{
    cell::OnceCell,
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    hash::Hash,
    net::SocketAddr,
};

use argon2::Argon2;
use password_hash::{PasswordHashString, PasswordHasher, SaltString};
use rand::rngs::OsRng;

use crate::state::{
    ID,
    error::{PotMutationError, RoomMutationError, StateMutationError},
    room::{pot::Pot, wager::Wager},
};

use super::error::AdminRequestError;

mod member_state;
pub mod pot;
pub mod wager;
pub use member_state::MemberState;

pub struct Room {
    room_code: RoomCode,
    members: HashMap<ID, MemberState>,
    address_map: HashMap<SocketAddr, ID>,
    next_member_id: ID,
    admin_pass_hash: password_hash::PasswordHashString,
    admin_pass_salt: password_hash::SaltString,
    admins: HashSet<SocketAddr>,
    pots: HashMap<ID, Pot>,
    next_pot_id: usize,
    wagers: HashMap<ID, Wager>,
    next_wager_id: usize,
}
impl Room {
    const HASHER: OnceCell<Argon2<'_>> = OnceCell::new();
    pub fn new(room_code: RoomCode, admin_pass: &str) -> Self {
        let (hash, salt) = Room::hash_pass(admin_pass);
        Self {
            room_code,
            members: HashMap::new(),
            next_member_id: 0,
            address_map: HashMap::new(),
            admins: HashSet::new(),
            admin_pass_hash: hash,
            admin_pass_salt: salt,
            pots: HashMap::new(),
            next_pot_id: 0,
            wagers: HashMap::new(),
            next_wager_id: 0,
        }
    }
    pub fn get_addresses(&self) -> impl Iterator<Item = &SocketAddr> {
        self.address_map.keys()
    }

    pub fn add_user(
        &mut self,
        user_addr: SocketAddr,
        name: String,
    ) -> Result<ID, RoomMutationError> {
        if self.address_map.contains_key(&user_addr) {
            Err(RoomMutationError::UserAlreadyExists(
                user_addr,
                self.room_code,
            ))
        } else {
            let id = self.next_member_id.clone();
            self.address_map.insert(user_addr, id);
            self.members.insert(id, MemberState::new(name, id));
            self.next_member_id += 1;
            Ok(id)
        }
    }
    pub fn remove_user(&mut self, user: ID) -> Result<(), RoomMutationError> {
        if let Some(mut user_state) = self.members.remove(&user) {
            for wager_id in user_state.current_wagers_mut().iter() {
                if let Some(wager) = self.wagers.get_mut(wager_id) {
                    wager.remove_user(user);
                }
            }
            for pot_id in user_state.current_pots_mut().iter() {
                if let Some(pot) = self.pots.get_mut(pot_id) {
                    let _ = pot.remove_user(user);
                }
            }
            if let Some(addr) = self
                .address_map
                .iter()
                .find(|(_, id)| **id == user)
                .map(|(addr, _)| *addr)
            {
                self.address_map.remove(&addr);
            }

            Ok(())
        } else {
            Err(RoomMutationError::UserNotInRoom(user, self.room_code))
        }
    }
    fn hash_pass(pass: &str) -> (password_hash::PasswordHashString, SaltString) {
        let salt = SaltString::generate(&mut OsRng);
        (Self::hash_pass_with_salt(pass, &salt), salt)
    }
    fn hash_pass_with_salt(pass: &str, salt: &SaltString) -> PasswordHashString {
        let hasher = Self::HASHER;
        let hasher = hasher.get_or_init(|| Argon2::default());
        hasher
            .hash_password(pass.as_bytes(), salt)
            .expect("Hashing should succeed!")
            .into()
    }
    fn check_admin_pass(&self, pass: &str) -> bool {
        log::trace!(
            "Checking admin password: {pass}, {:?}",
            self.admin_pass_hash
        );
        self.admin_pass_hash == Self::hash_pass_with_salt(pass, &self.admin_pass_salt)
    }
    pub fn add_admin(&mut self, user: SocketAddr, pass: &str) -> Result<(), AdminRequestError> {
        if self.check_admin_pass(pass) {
            if self.admins.insert(user) {
                Ok(())
            } else {
                Err(AdminRequestError::AlreadyAdmin)
            }
        } else {
            Err(AdminRequestError::IncorrectPassword)
        }
    }
    pub fn bless_score(&mut self, to: &ID, amount: i64) -> Result<(ID, i64), RoomMutationError> {
        let user = self
            .members
            .get_mut(to)
            .ok_or(RoomMutationError::UserNotInRoom(*to, self.room_code))?;
        user.set_score(user.score() + amount)?;
        Ok((*to, user.score()))
    }
    pub fn transfer_score(
        &mut self,
        from: &ID,
        to: &ID,
        amount: i64,
    ) -> Result<((ID, i64), (ID, i64)), RoomMutationError> {
        if !self.members.contains_key(to) {
            return Err(RoomMutationError::UserNotInRoom(*to, self.room_code));
        }
        let from_state = self
            .members
            .get_mut(&from)
            .ok_or(RoomMutationError::UserNotInRoom(*from, self.room_code))?;
        let prev_from_score = from_state.score();
        from_state.set_score(from_state.score() - amount)?;
        let from_score = from_state.score().clone();
        let to_state = self
            .members
            .get_mut(&to)
            .expect("Already checked if key contained!");

        if let Err(e) = to_state.set_score(to_state.score() + amount) {
            self.members
                .get_mut(from)
                .expect("Already checked from exists!")
                .set_score(prev_from_score)
                .expect("Prev from score should always be valid!");
            return Err(e);
        }

        let to_score = to_state.score().clone();
        Ok(((*from, from_score), (*to, to_score)))
    }
    pub fn create_pot(&mut self, score_requirement: i64, desc: String) -> Pot {
        let id = self.next_pot_id.clone();
        self.next_pot_id += 1;
        self.pots.insert(id, Pot::new(id, score_requirement, desc));
        self.pots.get(&id).unwrap().clone()
    }
    pub fn add_user_to_pot(&mut self, user_id: ID, pot_id: ID) -> Result<i64, StateMutationError> {
        //alas, no partial borrows
        let room_code = self.room_code;
        let score_req = self
            .pots
            .get(&pot_id)
            .ok_or(RoomMutationError::NonexistentPot(pot_id, room_code))?
            .score_req();

        let user = self
            .members
            .get_mut(&user_id)
            .ok_or(RoomMutationError::UserNotInRoom(user_id, room_code))?;

        if user.score() - score_req < 0 {
            {
                return Err(PotMutationError::InsufficientScore {
                    user_id,
                    pot_id,
                    user_score: user.score(),
                    score_req: score_req,
                }
                .into());
            }
        }

        self.pots
            .get_mut(&pot_id)
            .expect("We've already got the pot!")
            .join(user_id)?;
        user.set_score(user.score() - score_req).unwrap();
        user.current_pots_mut().insert(pot_id);
        Ok(user.score())
    }
    pub fn resolve_pot(
        &mut self,
        pot_id: ID,
        winner_id: ID,
    ) -> Result<(ID, i64), StateMutationError> {
        let pot = self
            .pots
            .get(&pot_id)
            .ok_or(RoomMutationError::NonexistentPot(pot_id, self.room_code))?;
        let winner = self
            .members
            .get_mut(&winner_id)
            .ok_or(RoomMutationError::UserNotInRoom(winner_id, self.room_code))?;
        winner.set_score(winner.score() + pot.resolve())?;
        let winner_score = winner.score();
        for id in pot.participants() {
            if let Some(participant) = self.members.get_mut(id) {
                participant.current_pots_mut().remove(&pot_id);
            }
        }
        self.pots.remove(&pot_id);

        Ok((winner_id, winner_score))
    }
    pub fn create_wager(
        &mut self,
        description: String,
        outcomes: Vec<wager::WagerOutcome>,
    ) -> Wager {
        let id = self.next_wager_id.clone();
        self.next_wager_id += 1;
        self.wagers
            .insert(id, Wager::new(id, description, outcomes));
        self.wagers.get(&id).expect("Just created wager!").clone()
    }
    pub fn add_user_to_wager(
        &mut self,
        wager_id: ID,
        user_id: ID,
        outcome_id: ID,
        amount: i64,
    ) -> Result<i64, StateMutationError> {
        self.wagers
            .get_mut(&wager_id)
            .ok_or(RoomMutationError::NonexistentWager {
                wager_id,
                room_code: self.room_code,
            })?
            .join(user_id, outcome_id, amount)?;
        let user = self
            .members
            .get_mut(&user_id)
            .ok_or(RoomMutationError::UserNotInRoom(user_id, self.room_code))?;
        user.set_score(user.score() - amount)?;
        user.current_wagers_mut().insert(wager_id);
        Ok(user.score())
    }
    pub fn resolve_wager(
        &mut self,
        wager_id: ID,
        outcome_id: ID,
    ) -> Result<Vec<(ID, i64)>, StateMutationError> {
        let results = self
            .wagers
            .get_mut(&wager_id)
            .ok_or(RoomMutationError::NonexistentWager {
                wager_id,
                room_code: self.room_code,
            })?
            .resolve(outcome_id)?;
        let mut out = Vec::new();
        if let Err(nonexistent) = results.iter().try_fold((), |_, next| {
            self.members
                .contains_key(&next.participant)
                .then(|| ())
                .ok_or(next.participant)
        }) {
            return Err(RoomMutationError::UserNotInRoom(nonexistent, self.room_code).into());
        }

        for result in results {
            let user = self
                .members
                .get_mut(&result.participant)
                .expect("Already asserted existence of user");

            user.set_score(user.score() + result.score_diff)
                .expect("Score diff should never be negative!");
            user.current_wagers_mut().remove(&wager_id);
            out.push((user.id(), user.score()));
        }
        self.wagers.remove(&wager_id);
        Ok(out)
    }
    pub fn code(&self) -> RoomCode {
        self.room_code
    }
    pub fn id_lookup(&self, addr: &SocketAddr) -> Option<ID> {
        self.address_map.get(addr).copied()
    }
    pub fn is_admin(&self, addr: &SocketAddr) -> bool {
        self.admins.contains(addr)
    }
    pub fn get_sync_data(&self) -> (Vec<MemberState>, Vec<Pot>, Vec<Wager>) {
        (
            self.members.clone().into_values().collect(),
            self.pots.clone().into_values().collect(),
            self.wagers.clone().into_values().collect(),
        )
    }
}

impl Debug for Room {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Room: \n   Room Code: {:?}\n     Members: {:?}\n   Address Map: {:?}\n     Admins: {:?}\n    Pots: {:?}\n    Wagers: {:?}\n    Next Member Id: {:?} \n     Next Pot Id: {:?} \n    Next Wager Id : {:?}",
            self.room_code,
            self.members,
            self.address_map,
            self.admins,
            self.pots,
            self.wagers,
            self.next_member_id,
            self.next_pot_id,
            self.next_wager_id
        )

        // f.debug_struct("Room")
        //     .field("room_code", &self.room_code)
        //     .field("members", &self.members)
        //     .field("address_map", &self.address_map)
        //     .field("next_member_id", &self.next_member_id)
        //     .field("admins", &self.admins)
        //     .field("pots", &self.pots)
        //     .field("next_pot_id", &self.next_pot_id)
        //     .field("wagers", &self.wagers)
        //     .field("next_wager_id", &self.next_wager_id)
        //     .finish()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct RoomCode([char; 8]);
impl From<[char; 8]> for RoomCode {
    fn from(value: [char; 8]) -> Self {
        RoomCode(value)
    }
}

impl Debug for RoomCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}
impl Display for RoomCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().collect::<String>())
    }
}

impl From<&RoomCode> for String {
    fn from(val: &RoomCode) -> Self {
        String::from_iter(val.0.iter())
    }
}
