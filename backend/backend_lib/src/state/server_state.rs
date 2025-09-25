use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use error::MessageHandleError;
use itertools::Itertools;
use parking_lot::RwLock;

use crate::connection::WSMessage;
use crate::connection::error::MessageSendError;
use crate::connection::message::{ClientMessage, Destination, ServerMessage};
use crate::state::error::RoomMutationError;

use super::room::RoomCode;
use super::*;

type LockedMap<K, V> = RwLock<HashMap<K, RwLock<V>>>;
type Locked2Map<K, V1, V2> = RwLock<HashMap<K, (RwLock<V1>, RwLock<V2>)>>;

pub struct ServerState {
    rooms: LockedMap<RoomCode, Room>,
    sessions: Locked2Map<SocketAddr, SessionData, crate::connection::Tx>,
}
impl ServerState {
    pub fn new() -> Arc<RwLock<ServerState>> {
        let state = Self {
            rooms: HashMap::new().into(),
            sessions: HashMap::new().into(),
        };
        Arc::new(RwLock::new(state))
    }

    pub fn add_room(&self, code: RoomCode, admin_pass: String) -> Result<(), MessageHandleError> {
        if self.rooms.read().contains_key(&code) {
            Err(MessageHandleError::RoomAlreadyExists(code))
        } else {
            self.rooms
                .write()
                .insert(code, RwLock::new(Room::new(code, &admin_pass)));

            Ok(())
        }
    }
    pub fn delete_room(&self, code: RoomCode) -> Result<(), MessageHandleError> {
        self.rooms
            .write()
            .remove(&code)
            .map(|_| ())
            .ok_or(MessageHandleError::NonexistentRoom(code))
    }
    pub fn add_user_to_room(
        &self,
        code: RoomCode,
        user: SocketAddr,
        name: String,
    ) -> Result<ID, MessageHandleError> {
        match self.rooms.read().get(&code) {
            Some(room) => {
                let id = room.write().add_user(user, name)?;
                self.sessions
                    .read()
                    .get(&user)
                    .ok_or(MessageHandleError::NonexistentSession(user))?
                    .0
                    .write()
                    .set_current_room(code);
                Ok(id)
            }
            None => Err(MessageHandleError::NonexistentRoom(code)),
        }
    }

    pub fn init_session(&self, addr: SocketAddr, tx: crate::connection::Tx) {
        let session_data = RwLock::new(SessionData::new());
        self.sessions
            .write()
            .insert(addr, (session_data, RwLock::new(tx)));
    }
    pub fn cleanup_session(&mut self, addr: &SocketAddr) {
        log::trace!("starting cleanup for addr:{addr}");
        if let Some((sess, _)) = self.sessions.write().remove(addr) {
            let rooms = self.rooms.read();
            if let Some(room) = sess.read().current_room().and_then(|room| rooms.get(&room)) {
                let mut room = room.upgradable_read();
                if let Some(id) = room.id_lookup(addr) {
                    log::trace!("Removing user: {id} from room {}", room.code());
                    let _ = room
                        .with_upgraded(|room| room.remove_user(id))
                        .inspect_err(|e| log::error!("{e}"));
                }
            }
        }
        log::trace!("finished cleaning up for addr:{addr}")
    }

    fn send_ws_message(&self, addr: &SocketAddr, msg: WSMessage) -> Result<(), MessageSendError> {
        self.sessions
            .read()
            .get(addr)
            .ok_or(MessageSendError::NonexistentSession(*addr))?
            .1
            .write()
            .unbounded_send(msg)
            .map_err(Into::into)
    }

    pub fn send_to_addr(
        &self,
        addr: &SocketAddr,
        msg: impl TryInto<WSMessage, Error = impl Into<MessageSendError>>,
    ) -> Result<(), MessageSendError> {
        self.send_ws_message(addr, msg.try_into().map_err(Into::into)?)
    }
    pub fn send_to_peers(
        &self,
        user: &SocketAddr,
        msg: impl TryInto<WSMessage, Error = impl Into<MessageSendError>>,
        inclusive: bool,
    ) -> Result<(), Vec<MessageSendError>> {
        let sessions = self.sessions.read();
        let (user_sess_data, _) = sessions
            .get(user)
            .ok_or(vec![MessageSendError::NonexistentSession(*user)])?;
        let user_room = user_sess_data
            .read()
            .current_room()
            .ok_or(vec![MessageSendError::UserNotInRoom])?;
        let peers = self
            .rooms
            .read()
            .get(&user_room)
            .ok_or(vec![MessageSendError::NonexistentRoom(user_room)])?
            .read()
            .get_addresses()
            .filter_map(|x| {
                if (x != user) || inclusive {
                    Some(*x)
                } else {
                    None
                }
            })
            .collect_vec();
        let message: WSMessage = msg.try_into().map_err(|x| vec![x.into()])?;
        let mut errors = Vec::new();
        for peer in peers {
            if let Err(e) = self.send_ws_message(&peer, message.clone()) {
                errors.push(e);
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    pub fn send_to_everyone(
        &self,
        msg: impl TryInto<WSMessage, Error = impl Into<MessageSendError>>,
    ) -> Result<(), Vec<MessageSendError>> {
        let sessions = self.sessions.read();
        let message: WSMessage = msg.try_into().map_err(|x| vec![x.into()])?;
        let mut errors = Vec::new();
        for addr in sessions.keys() {
            if let Err(e) = self.send_ws_message(addr, message.clone()) {
                errors.push(e)
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn handle_message(
        &self,
        message: ClientMessage,
        sender: SocketAddr,
    ) -> Result<Vec<(ServerMessage, Destination)>, MessageHandleError> {
        match message {
            ClientMessage::CreateRoom { code, admin_pass } => {
                self.add_room(code, admin_pass)?;
                Ok(Vec::new())
            }
            ClientMessage::JoinRoom {
                code: room_code,
                name,
            } => {
                let id = self.add_user_to_room(room_code, sender, name.clone())?;
                let (members, pots, wager) = self
                    .rooms
                    .read()
                    .get(&room_code)
                    .ok_or(MessageHandleError::NonexistentRoom(room_code))?
                    .read()
                    .get_sync_data();
                Ok(vec![
                    (
                        ServerMessage::UserJoined { name, id },
                        Destination::PeersInclusive,
                    ),
                    (
                        ServerMessage::SynchronizeRoom {
                            members,
                            pots,
                            wager,
                            requester_id: id,
                        },
                        Destination::Myself,
                    ),
                ])
            }
            ClientMessage::LeaveRoom { room_code } => {
                let rooms = self.rooms.read();
                let mut room = rooms
                    .get(&room_code)
                    .ok_or(MessageHandleError::NonexistentRoom(room_code))?
                    .write();
                let id = room
                    .id_lookup(&sender)
                    .ok_or(RoomMutationError::AddressNotInRoom(sender, room_code))?;
                room.remove_user(id)?;
                Ok(vec![
                    (
                        ServerMessage::UserRemoved { id },
                        Destination::PeersExclusive,
                    ),
                    (ServerMessage::RecieverLeft, Destination::Myself),
                ])
            }
            ClientMessage::RemoveFromRoom {
                code: room_code,
                id: removed_id,
            } => {
                let addr = self
                    .rooms
                    .read()
                    .get(&room_code)
                    .ok_or(MessageHandleError::NonexistentRoom(room_code))?
                    .write()
                    .remove_user(removed_id)?;
                Ok(vec![
                    (
                        ServerMessage::UserRemoved { id: removed_id },
                        Destination::PeersInclusive,
                    ),
                    (ServerMessage::RecieverLeft, Destination::Specific(addr)),
                ])
            }
            ClientMessage::DeleteRoom { room_code } => {
                self.delete_room(room_code)?;
                Ok(vec![(
                    ServerMessage::RoomDeleted,
                    Destination::PeersInclusive,
                )])
            }
            ClientMessage::RequestAdmin { room, password } => {
                let rooms = self.rooms.read();
                let mut room = rooms
                    .get(&room)
                    .ok_or(MessageHandleError::NonexistentRoom(room))?
                    .write();
                room.id_lookup(&sender)
                    .ok_or(RoomMutationError::AddressNotInRoom(sender, room.code()))?;
                room.add_admin(sender, &password)?;

                Ok(vec![(ServerMessage::AdminGranted, Destination::Myself)])
            }
            ClientMessage::TransferScore { from, to, amount } => {
                let rooms = self.rooms.read();
                let room_code = self.get_users_room(&sender)?;
                let (from_info, to_info) = rooms
                    .get(&room_code)
                    .ok_or(MessageHandleError::NonexistentRoom(room_code))?
                    .write()
                    .transfer_score(&from, &to, amount)?;
                Ok(vec![
                    (
                        ServerMessage::ScoreChanged {
                            user_id: from_info.0,
                            new_amount: from_info.1,
                        },
                        Destination::PeersInclusive,
                    ),
                    (
                        ServerMessage::ScoreChanged {
                            user_id: to_info.0,
                            new_amount: to_info.1,
                        },
                        Destination::PeersInclusive,
                    ),
                ])
            }
            ClientMessage::GiveScore { to, amount } => {
                if amount < 0 {
                    return Err(RoomMutationError::NegativeScore.into());
                }
                let rooms = self.rooms.read();
                let room_code = self.get_users_room(&sender)?;
                let mut room = rooms
                    .get(&room_code)
                    .ok_or(MessageHandleError::NonexistentRoom(room_code))?
                    .write();

                let from = room
                    .id_lookup(&sender)
                    .ok_or(RoomMutationError::AddressNotInRoom(sender, room_code))?;

                let (from_info, to_info) = room.transfer_score(&from, &to, amount)?;
                Ok(vec![
                    (
                        ServerMessage::ScoreChanged {
                            user_id: from_info.0,
                            new_amount: from_info.1,
                        },
                        Destination::PeersInclusive,
                    ),
                    (
                        ServerMessage::ScoreChanged {
                            user_id: to_info.0,
                            new_amount: to_info.1,
                        },
                        Destination::PeersInclusive,
                    ),
                ])
            }
            ClientMessage::BlessScore { to, amount } => {
                let rooms = self.rooms.read();
                let room_code = self.get_users_room(&sender)?;
                let mut room = rooms
                    .get(&room_code)
                    .ok_or(MessageHandleError::NonexistentRoom(room_code))?
                    .write();
                let (user_id, new_amount) = room.bless_score(&to, amount)?;
                Ok(vec![(
                    ServerMessage::ScoreChanged {
                        user_id,
                        new_amount,
                    },
                    Destination::PeersInclusive,
                )])
            }
            ClientMessage::RemoveScore { from, amount } => {
                let rooms = self.rooms.read();
                let room_code = self.get_users_room(&sender)?;
                let mut room = rooms
                    .get(&room_code)
                    .ok_or(MessageHandleError::NonexistentRoom(room_code))?
                    .write();
                let (user_id, new_amount) = room.bless_score(&from, -amount)?;
                Ok(vec![(
                    ServerMessage::ScoreChanged {
                        user_id,
                        new_amount,
                    },
                    Destination::PeersInclusive,
                )])
            }
            ClientMessage::CreatePot {
                room_code,
                score_requirement,
                description,
            } => {
                let pot = self
                    .rooms
                    .read()
                    .get(&room_code)
                    .ok_or(MessageHandleError::NonexistentRoom(room_code))?
                    .write()
                    .create_pot(score_requirement, description);
                Ok(vec![(
                    ServerMessage::PotCreated { pot },
                    Destination::PeersInclusive,
                )])
            }
            ClientMessage::JoinPot { pot_id, room_code } => {
                let rooms = self.rooms.read();
                let room = rooms
                    .get(&room_code)
                    .ok_or(MessageHandleError::NonexistentRoom(room_code))?;
                let user_id = room
                    .read()
                    .id_lookup(&sender)
                    .ok_or(RoomMutationError::AddressNotInRoom(sender, room_code))?;
                let new_amount = room.write().add_user_to_pot(user_id, pot_id)?;
                Ok(vec![
                    (
                        ServerMessage::PotJoined { pot_id, user_id },
                        Destination::PeersInclusive,
                    ),
                    (
                        ServerMessage::ScoreChanged {
                            user_id,
                            new_amount,
                        },
                        Destination::PeersInclusive,
                    ),
                ])
            }

            ClientMessage::ResolvePot {
                pot_id,
                winner,
                room_id,
            } => {
                let (winner_id, new_score) = self
                    .rooms
                    .read()
                    .get(&room_id)
                    .ok_or(MessageHandleError::NonexistentRoom(room_id))?
                    .write()
                    .resolve_pot(pot_id, winner)?;
                Ok(vec![
                    (
                        ServerMessage::PotResolved { id: pot_id },
                        Destination::PeersInclusive,
                    ),
                    (
                        ServerMessage::ScoreChanged {
                            user_id: winner_id,
                            new_amount: new_score,
                        },
                        Destination::PeersInclusive,
                    ),
                ])
            }
            ClientMessage::CreateWager {
                name,
                outcomes,
                room_id,
            } => {
                let wager = self
                    .rooms
                    .read()
                    .get(&room_id)
                    .ok_or(MessageHandleError::NonexistentRoom(room_id))?
                    .write()
                    .create_wager(name, outcomes);
                Ok(vec![(
                    ServerMessage::WagerCreated { wager },
                    Destination::PeersInclusive,
                )])
            }
            ClientMessage::JoinWager {
                wager_id,
                outcome_id,
                room_id,
                amount,
            } => {
                if amount < 0 {
                    return Err(RoomMutationError::NegativeScore.into());
                }
                let rooms = self.rooms.read();
                let room = rooms
                    .get(&room_id)
                    .ok_or(MessageHandleError::NonexistentRoom(room_id))?;
                let user_id = room
                    .read()
                    .id_lookup(&sender)
                    .ok_or(RoomMutationError::AddressNotInRoom(sender, room_id))?;
                let new_score = room
                    .write()
                    .add_user_to_wager(wager_id, user_id, outcome_id, amount)?;
                Ok(vec![
                    (
                        ServerMessage::WagerJoined {
                            wager_id,
                            user_id,
                            outcome_id,
                            amount,
                        },
                        Destination::PeersInclusive,
                    ),
                    (
                        ServerMessage::ScoreChanged {
                            user_id,
                            new_amount: new_score,
                        },
                        Destination::PeersInclusive,
                    ),
                ])
            }
            ClientMessage::ResolveWager {
                wager_id,
                outcome_id,
                room_id,
            } => {
                let mut msgs = self
                    .rooms
                    .read()
                    .get(&room_id)
                    .ok_or(MessageHandleError::NonexistentRoom(room_id))?
                    .write()
                    .resolve_wager(wager_id, outcome_id)?
                    .into_iter()
                    .map(|(user_id, new_amount)| {
                        (
                            ServerMessage::ScoreChanged {
                                user_id,
                                new_amount,
                            },
                            Destination::PeersInclusive,
                        )
                    })
                    .collect_vec();
                msgs.push((
                    ServerMessage::WagerResolved { id: wager_id },
                    Destination::PeersInclusive,
                ));
                Ok(msgs)
            }
            ClientMessage::Debug => {
                log::debug!("{self:?}");
                Ok(Vec::new())
            }
        }
    }
    fn get_users_room(&self, user: &SocketAddr) -> Result<RoomCode, MessageHandleError> {
        self.sessions
            .read()
            .get(user)
            .ok_or(MessageHandleError::NonexistentSession(*user))?
            .0
            .read()
            .current_room()
            .ok_or(MessageHandleError::UserNotInAnyRoom(*user))
    }
    pub fn is_user_admin(
        &self,
        user: &SocketAddr,
        room_code: Option<RoomCode>,
    ) -> Result<bool, MessageHandleError> {
        let room_code = room_code.unwrap_or(
            self.sessions
                .read()
                .get(user)
                .ok_or(MessageHandleError::NonexistentSession(*user))?
                .0
                .read()
                .current_room()
                .ok_or(MessageHandleError::UserNotInAnyRoom(*user))?,
        );
        Ok(self
            .rooms
            .read()
            .get(&room_code)
            .ok_or(MessageHandleError::NonexistentRoom(room_code))?
            .read()
            .is_admin(user))
    }
}

impl std::fmt::Debug for ServerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ServerState: \n Rooms: {} \n -------- \n Sessions: {}",
            self.rooms
                .read()
                .iter()
                .map(|(code, room)| format!("   {code} -> {:?}", room.read()))
                .join("\n -- \n"),
            self.sessions
                .read()
                .iter()
                .map(|(addr, data)| format!("   {addr} -> {:?}", data.0.read()))
                .join("\n --- \n")
        )
    }
}
