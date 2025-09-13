use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use super::{WSMessage, error::MessageParseError};
use crate::state::{
    ID,
    room::{
        MemberState, RoomCode,
        pot::Pot,
        wager::{Wager, WagerOutcome},
    },
};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Destination {
    Myself,
    PeersExclusive,
    PeersInclusive,
    Specific(SocketAddr),
    Everyone,
}

//These enums should contain values that update the state of the client accordingly
#[derive(Serialize, Clone, PartialEq, Eq, Debug, schemars::JsonSchema)]
pub enum ServerMessage {
    SynchronizeRoom {
        members: Vec<MemberState>,
        pots: Vec<Pot>,
        wager: Vec<Wager>,
    },
    UserJoined {
        name: String,
        id: ID,
    },
    RoomDeleted,
    UserRemoved(ID),
    PotCreated(Pot),
    PotJoined {
        pot_id: ID,
        user_id: ID,
    },
    PotResolved(ID),
    WagerCreated(Wager),
    WagerJoined {
        wager_id: ID,
        user_id: ID,
        outcome_id: ID,
        amount: i64,
    },
    WagerResolved(ID),
    ScoreChanged {
        user_id: ID,
        new_amount: i64,
    },
    AdminGranted,

    Error {
        description: String,
        display_to_user: bool,
    },
}

impl TryInto<WSMessage> for ServerMessage {
    type Error = serde_json::Error;
    fn try_into(self) -> Result<WSMessage, serde_json::Error> {
        Ok(WSMessage::from(serde_json::to_string(&self)?))
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, schemars::JsonSchema)]
pub enum ClientMessage {
    CreateRoom {
        code: RoomCode,
        admin_pass: String,
    },
    JoinRoom(RoomCode, String),
    LeaveRoom(RoomCode),
    RemoveFromRoom(RoomCode, ID),
    DeleteRoom(RoomCode),
    RequestAdmin {
        room: RoomCode,
        password: String,
    },
    BlessScore {
        to: ID,
        amount: i64,
    },
    RemoveScore {
        from: ID,
        amount: i64,
    },
    GiveScore {
        to: ID,
        amount: i64,
    },
    TransferScore {
        from: ID,
        to: ID,
        amount: i64,
    },
    CreatePot {
        room_code: RoomCode,
        score_requirement: i64,
        description: String,
    },

    JoinPot {
        room_code: RoomCode,
        pot_id: ID,
    },

    ResolvePot {
        room_id: RoomCode,
        pot_id: ID,
        winner: ID,
    },
    CreateWager {
        room_id: RoomCode,
        name: String,
        outcomes: Vec<WagerOutcome>,
    },
    JoinWager {
        room_id: RoomCode,
        wager_id: ID,
        outcome_id: ID,
        amount: i64,
    },
    ResolveWager {
        room_id: RoomCode,
        wager_id: ID,
        outcome_id: ID,
    },
}

impl ClientMessage {
    pub fn requires_admin(&self) -> Option<Option<RoomCode>> {
        match self {
            ClientMessage::CreateRoom {
                code: _,
                admin_pass: _,
            } => None,
            ClientMessage::JoinRoom(_, _) => None,
            ClientMessage::LeaveRoom(_) => None,
            ClientMessage::RemoveFromRoom(code, _) => Some(Some(*code)),
            ClientMessage::DeleteRoom(code) => Some(Some(*code)),
            ClientMessage::RequestAdmin {
                room: _,
                password: _,
            } => None,
            ClientMessage::BlessScore { to: _, amount: _ } => Some(None),
            ClientMessage::RemoveScore { from: _, amount: _ } => Some(None),
            ClientMessage::GiveScore { to: _, amount: _ } => None,
            ClientMessage::TransferScore {
                from: _,
                to: _,
                amount: _,
            } => None,
            ClientMessage::CreatePot {
                room_code,
                score_requirement: _,
                description: _,
            } => Some(Some(*room_code)),
            ClientMessage::JoinPot {
                room_code: _,
                pot_id: _,
            } => None,
            ClientMessage::ResolvePot {
                room_id,
                pot_id: _,
                winner: _,
            } => Some(Some(*room_id)),
            ClientMessage::CreateWager {
                room_id,
                name: _,
                outcomes: _,
            } => Some(Some(*room_id)),
            ClientMessage::JoinWager {
                room_id: _,
                wager_id: _,
                outcome_id: _,
                amount: _,
            } => None,
            ClientMessage::ResolveWager {
                room_id,
                wager_id: _,
                outcome_id: _,
            } => Some(Some(*room_id)),
        }
    }
}
impl TryFrom<WSMessage> for ClientMessage {
    type Error = MessageParseError;
    fn try_from(value: WSMessage) -> Result<Self, Self::Error> {
        let text = value.to_text()?;
        serde_json::from_str(text).map_err(Into::into)
    }
}
