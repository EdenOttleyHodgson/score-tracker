use std::{net::SocketAddr, sync::Arc};

use futures::channel::mpsc::unbounded;
use itertools::Itertools;
use parking_lot::RwLock;
use rstest::fixture;

use crate::{
    connection::{
        Rx,
        message::{ClientMessage, Destination, ServerMessage},
    },
    state::{ServerState, error::MessageHandleError, room::RoomCode},
};

mod message_handling;
mod state_manipulation;

#[allow(dead_code)]
struct MockConnection {
    rx: Rx,
    recieved: Vec<(ServerMessage, Destination)>,
    errors: Vec<MessageHandleError>,
    state: Arc<RwLock<ServerState>>,
    addr: SocketAddr,
}
impl MockConnection {
    fn new(state: Arc<RwLock<ServerState>>, addr: SocketAddr) -> Self {
        let (tx, rx) = unbounded();
        state.write().init_session(addr, tx);
        Self {
            rx,
            recieved: Vec::new(),
            errors: Vec::new(),
            state,
            addr,
        }
    }
    fn send_message_setup(&mut self, msg: ClientMessage) {
        self.state
            .read()
            .handle_message(msg.clone(), self.addr)
            .expect(&format!(
                "Message sending failed during test setup: {msg:?}"
            ));
    }
    fn send_message(&mut self, msg: ClientMessage) {
        match self.state.read().handle_message(msg, self.addr) {
            Ok(mut res) => self.recieved.append(&mut res),
            Err(err) => self.errors.push(err),
        }
    }
}

type StateFixture = (Arc<RwLock<ServerState>>, Vec<MockConnection>);
#[fixture]
fn no_client_state() -> StateFixture {
    (ServerState::new(), vec![])
}
#[fixture]
fn single_client_state() -> StateFixture {
    let state = ServerState::new();
    let connection = MockConnection::new(state.clone(), "127.0.0.1:8080".parse().unwrap());
    (state, vec![connection])
}
#[fixture]
fn multi_client_state() -> StateFixture {
    let state = ServerState::new();
    let mut connections = Vec::new();
    let v4addr = std::net::Ipv4Addr::new(127, 0, 0, 1);
    for portnum in 8080..=8085 {
        connections.push(MockConnection::new(
            state.clone(),
            SocketAddr::new(v4addr.into(), portnum),
        ));
    }
    (state, connections)
}

impl From<&'static str> for RoomCode {
    fn from(value: &'static str) -> Self {
        value.chars().take(8).collect_array().unwrap().into()
    }
}
