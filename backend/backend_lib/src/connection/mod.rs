//ss
pub mod error;
pub mod message;
use std::{net::SocketAddr, sync::Arc};

use error::*;
use futures::{
    StreamExt, TryStreamExt,
    channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded},
    future, pin_mut,
    stream::{SplitSink, SplitStream},
};
use itertools::Itertools;
use message::{ClientMessage, ServerMessage};
use parking_lot::RwLock;
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;

use crate::{
    connection::message::Destination,
    state::{ServerState, error::MessageHandleError},
};
pub type WSStream = WebSocketStream<TcpStream>;
pub type WSMessage = tokio_tungstenite::tungstenite::Message;
pub type TungsteniteError = tokio_tungstenite::tungstenite::Error;

pub type Tx = UnboundedSender<WSMessage>;
pub type Rx = UnboundedReceiver<WSMessage>;
type Sink = SplitSink<WSStream, WSMessage>;
type Stream = SplitStream<WSStream>;

pub struct Connection {
    addr: SocketAddr,
    rx: Rx,
    sink: Sink,
    stream: Stream,
    server_state: Arc<RwLock<ServerState>>,
}

impl Connection {
    pub fn new(
        addr: SocketAddr,
        ws_stream: WSStream,
        server_state: Arc<RwLock<ServerState>>,
    ) -> Self {
        let (tx, rx): (Tx, Rx) = unbounded();
        let (sink, stream) = ws_stream.split();
        server_state.write().init_session(addr, tx);
        Self {
            addr,
            rx,
            sink,
            stream: stream,
            server_state,
        }
    }
    pub async fn handle_connection(self) {
        let handle_message = self.stream.try_for_each(async |msg| {
            log::trace!("Message recieved!: {msg}");
            match ClientMessage::try_from(msg) {
                Ok(msg) => {
                    if let Some(room_code) = msg.requires_admin() {
                        match self
                            .server_state
                            .read()
                            .is_user_admin(&self.addr, room_code)
                        {
                            Ok(is_admin) => {
                                if !is_admin {
                                    Self::forward_error_to_client(
                                        self.server_state.clone(),
                                        self.addr,
                                        MessageHandleError::AuthorizationError,
                                    );
                                    return Ok(());
                                }
                            }
                            Err(e) => {
                                Self::forward_error_to_client(
                                    self.server_state.clone(),
                                    self.addr,
                                    e,
                                );
                                return Ok(());
                            }
                        }
                    }
                    let errors = match self.server_state.read().handle_message(msg, self.addr) {
                        Ok(responses) => responses
                            .into_iter()
                            .filter_map(|(response, dest)| {
                                if let Err(es) = Self::send_message(
                                    self.server_state.clone(),
                                    &self.addr,
                                    response,
                                    dest,
                                ) {
                                    Some(es)
                                } else {
                                    None
                                }
                            })
                            .flatten()
                            .collect_vec(),
                        Err(error) => {
                            if Self::forward_error_to_client(
                                self.server_state.clone(),
                                self.addr,
                                error,
                            ) {
                                return Ok(());
                            } else {
                                vec![]
                            }
                        }
                    };
                    let mut break_flag = false;
                    for err in errors {
                        break_flag = break_flag
                            || Self::forward_error_to_client(
                                self.server_state.clone(),
                                self.addr,
                                err,
                            );
                    }
                    if break_flag {
                        return Ok(());
                    }
                }
                Err(e) => {
                    if Self::forward_error_to_client(self.server_state.clone(), self.addr, e) {
                        return Ok(());
                    }
                }
            }
            Ok(())
        });
        let forward = self.rx.map(Ok).forward(self.sink);
        pin_mut!(handle_message, forward);
        future::select(handle_message, forward).await;
        log::trace!("Connection terminated. Address: {}", self.addr);
        self.server_state.write().cleanup_session(&self.addr);
    }

    fn send_message(
        server_state: Arc<RwLock<ServerState>>,
        self_addr: &SocketAddr,
        msg: ServerMessage,
        dest: Destination,
    ) -> Result<(), Vec<MessageSendError>> {
        match dest {
            message::Destination::Myself => server_state
                .read()
                .send_to_addr(self_addr, msg)
                .map_err(|x| vec![x]),
            message::Destination::PeersExclusive => {
                server_state.read().send_to_peers(self_addr, msg, false)
            }
            message::Destination::PeersInclusive => {
                server_state.read().send_to_peers(self_addr, msg, true)
            }
            message::Destination::Specific(socket_addr) => server_state
                .read()
                .send_to_addr(&socket_addr, msg)
                .map_err(|x| vec![x]),
            message::Destination::Everyone => server_state.read().send_to_everyone(msg),
        }?;
        Ok(())
    }
    fn forward_error_to_client<E>(
        server_state: Arc<RwLock<ServerState>>,
        addr: SocketAddr,
        e: E,
    ) -> bool
    where
        E: Into<ServerMessage>,
        E: std::error::Error,
    {
        log::error!("Error handling message!: {e:?}");
        if let Err(e) = server_state.write().send_to_addr(&addr, e.into()) {
            log::error!("Error sending error message to client: {}!: {e}", addr);
            true
        } else {
            false
        }
    }
}
