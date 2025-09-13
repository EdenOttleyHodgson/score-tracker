use std::net::SocketAddr;

use thiserror::Error;

use crate::state::ID;

use super::room::RoomCode;

#[derive(Error, Debug, PartialEq)]
pub enum MessageHandleError {
    #[error("Failed to handle message, room: {0} doesn't exist!")]
    NonexistentRoom(RoomCode),
    #[error("Room with code {0} already exists")]
    RoomAlreadyExists(RoomCode),
    #[error("User from socket: {0} is not in a room")]
    UserNotInAnyRoom(SocketAddr),
    #[error("Session from socket: {0} does not exist")]
    NonexistentSession(SocketAddr),
    #[error("Could not mutate room: {0}")]
    RoomMutationError(RoomMutationError),
    #[error("Could not mutate pot: {0}")]
    PotMutationError(PotMutationError),
    #[error("Could not mutate wager: {0}")]
    WagerMutationError(WagerMutationError),
    #[error("This user is not an admin.")]
    AuthorizationError,
}

impl MessageHandleError {
    pub fn should_display(&self) -> bool {
        match self {
            MessageHandleError::NonexistentRoom(_) | MessageHandleError::RoomAlreadyExists(_) => {
                true
            }
            MessageHandleError::UserNotInAnyRoom(_) => true,
            MessageHandleError::NonexistentSession(_) => false,
            MessageHandleError::RoomMutationError(room_mutation_error) => match room_mutation_error
            {
                RoomMutationError::AdminRequestError(_) => true,
                RoomMutationError::UserAlreadyExists(_, _) => false,
                RoomMutationError::UserNotInRoom(_, _) => false,
                RoomMutationError::AddressNotInRoom(_, _) => false,
                RoomMutationError::NonexistentPot(_, _) => false,
                RoomMutationError::NegativeScore => true,
                RoomMutationError::NonexistentWager {
                    wager_id: _,
                    room_code: _,
                } => false,
            },
            MessageHandleError::PotMutationError(pot_mutation_error) => match pot_mutation_error {
                PotMutationError::UserAlreadyExists {
                    user_id: _,
                    pot_id: _,
                } => true,
                PotMutationError::UserNotInPot {
                    user_id: _,
                    pot_id: _,
                } => false,
                PotMutationError::InsufficientScore {
                    user_id: _,
                    pot_id: _,
                    user_score: _,
                    score_req: _,
                } => true,
            },
            MessageHandleError::WagerMutationError(wager_mutation_error) => {
                match wager_mutation_error {
                    WagerMutationError::UserAlreadyExists {
                        user_id: _,
                        wager_id: _,
                    } => false,
                    WagerMutationError::NonexistentOutcome {
                        outcome_id: _,
                        wager_id: _,
                    } => false,
                    WagerMutationError::UserAlreadyChose {
                        user_id: _,
                        outcome_id: _,
                    } => false,
                }
            }
            MessageHandleError::AuthorizationError => true,
        }
    }
}
impl From<AdminRequestError> for MessageHandleError {
    fn from(value: AdminRequestError) -> Self {
        Self::RoomMutationError(RoomMutationError::AdminRequestError(value))
    }
}

impl From<RoomMutationError> for MessageHandleError {
    fn from(v: RoomMutationError) -> Self {
        Self::RoomMutationError(v)
    }
}

impl From<PotMutationError> for MessageHandleError {
    fn from(v: PotMutationError) -> Self {
        Self::PotMutationError(v)
    }
}

impl From<WagerMutationError> for MessageHandleError {
    fn from(v: WagerMutationError) -> Self {
        Self::WagerMutationError(v)
    }
}

impl From<StateMutationError> for MessageHandleError {
    fn from(value: StateMutationError) -> Self {
        match value {
            StateMutationError::Room(e) => Self::RoomMutationError(e),
            StateMutationError::Pot(e) => Self::PotMutationError(e),
            StateMutationError::Wager(e) => Self::WagerMutationError(e),
        }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum AdminRequestError {
    #[error("Incorrect Password!")]
    IncorrectPassword,
    #[error("Already Administrator!")]
    AlreadyAdmin,
}

pub enum StateMutationError {
    Room(RoomMutationError),
    Pot(PotMutationError),
    Wager(WagerMutationError),
}

impl From<WagerMutationError> for StateMutationError {
    fn from(v: WagerMutationError) -> Self {
        Self::Wager(v)
    }
}

impl From<PotMutationError> for StateMutationError {
    fn from(v: PotMutationError) -> Self {
        Self::Pot(v)
    }
}

impl From<RoomMutationError> for StateMutationError {
    fn from(v: RoomMutationError) -> Self {
        Self::Room(v)
    }
}
#[derive(Error, Debug, PartialEq)]
pub enum RoomMutationError {
    #[error("Failed to make user admin of room!: {0}")]
    AdminRequestError(AdminRequestError),
    #[error("User with address: {0} already exists in room with code {1} ")]
    UserAlreadyExists(SocketAddr, RoomCode),
    #[error("User with ID: {0} is not in room with code: {1}")]
    UserNotInRoom(ID, RoomCode),
    #[error("User with Address: {0} is not in room with code: {1}")]
    AddressNotInRoom(SocketAddr, RoomCode),
    #[error("No pot with id: {0} exists in room with code {1}")]
    NonexistentPot(ID, RoomCode),
    #[error("todo")]
    NegativeScore,
    #[error("No wager with id: {wager_id} exists in room with code {room_code}")]
    NonexistentWager {
        wager_id: usize,
        room_code: RoomCode,
    },
}

impl From<AdminRequestError> for RoomMutationError {
    fn from(v: AdminRequestError) -> Self {
        Self::AdminRequestError(v)
    }
}
#[derive(Error, Debug, PartialEq)]
pub enum PotMutationError {
    #[error("User with id: {} already exists in pot with id {} ", .user_id, .pot_id)]
    UserAlreadyExists { user_id: ID, pot_id: ID },
    #[error("User with id: {} does not exist in pot with id {} ", .user_id, .pot_id)]
    UserNotInPot { user_id: ID, pot_id: ID },
    #[error(
        "User: {user_id} cannot join pot {pot_id} with score {user_score}, they need {score_req}"
    )]
    InsufficientScore {
        user_id: ID,
        pot_id: ID,
        user_score: i64,
        score_req: i64,
    },
}

#[derive(Error, Debug, PartialEq)]
pub enum WagerMutationError {
    #[error("User with id: {user_id} already exists in wager with id {wager_id} ")]
    UserAlreadyExists { user_id: ID, wager_id: ID },
    #[error("Outcome with id {outcome_id} does not exist in wager with id {wager_id}")]
    NonexistentOutcome { outcome_id: ID, wager_id: ID },
    #[error("User with id {user_id} already chose outcome {outcome_id}")]
    UserAlreadyChose { user_id: usize, outcome_id: usize },
}
