use std::net::SocketAddr;

use thiserror::Error;

use crate::state::{error::MessageHandleError, room::RoomCode};

use super::{WSMessage, message::ServerMessage};

#[derive(Error, Debug)]
pub enum MessageParseError {
    #[error("Message was not convertible to text, likely not UTF-8. {0}")]
    NotConvertibleToText(tokio_tungstenite::tungstenite::Error),
    #[error("Message could not be deserialized to a message: {0}")]
    DeserializationError(serde_json::Error),
}

impl From<serde_json::Error> for MessageParseError {
    fn from(v: serde_json::Error) -> Self {
        Self::DeserializationError(v)
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for MessageParseError {
    fn from(v: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::NotConvertibleToText(v)
    }
}
impl From<MessageParseError> for ServerMessage {
    fn from(val: MessageParseError) -> Self {
        ServerMessage::Error {
            description: format!("Server Side Message Parse Error: {val}"),
            display_to_user: false,
        }
    }
}
impl From<MessageHandleError> for ServerMessage {
    fn from(value: MessageHandleError) -> Self {
        ServerMessage::Error {
            description: format!("Error handling message: {value}"),
            display_to_user: value.should_display(),
        }
    }
}

#[derive(Error, Debug)]
pub enum MessageSendError {
    #[error("Could not communicate between channels server side: {0}")]
    TrySendError(futures::channel::mpsc::TrySendError<WSMessage>),
    #[error("Session with address: {0} does not exist.")]
    NonexistentSession(SocketAddr),
    #[error("Room with code: {0} does not exist.")]
    NonexistentRoom(RoomCode),
    #[error("Message could not be serialized into a websocket message: {0}")]
    MessageSerializationError(serde_json::Error),
    #[error("Message cannot be sent to peers, the sender isn't in a room!")]
    UserNotInRoom,
}
impl TryInto<super::TungsteniteError> for MessageSendError {
    fn try_into(self) -> Result<super::TungsteniteError, Self::Error> {
        match self {
            MessageSendError::TrySendError(try_send_error) => {
                if try_send_error.is_disconnected() {
                    Ok(super::TungsteniteError::AlreadyClosed)
                } else {
                    Err(())
                }
            }
            MessageSendError::NonexistentSession(_) => Ok(super::TungsteniteError::AlreadyClosed),
            MessageSendError::NonexistentRoom(_) => Ok(super::TungsteniteError::AlreadyClosed),
            MessageSendError::UserNotInRoom => Err(()),
            MessageSendError::MessageSerializationError(_) => Err(()),
        }
    }

    type Error = ();
}

impl From<serde_json::Error> for MessageSendError {
    fn from(v: serde_json::Error) -> Self {
        Self::MessageSerializationError(v)
    }
}

impl From<futures::channel::mpsc::TrySendError<WSMessage>> for MessageSendError {
    fn from(v: futures::channel::mpsc::TrySendError<WSMessage>) -> Self {
        Self::TrySendError(v)
    }
}
impl Into<ServerMessage> for MessageSendError {
    fn into(self) -> ServerMessage {
        ServerMessage::Error {
            description:
                "Internal Server Error, this is likely a bug, please contact the maintainer.".into(),
            display_to_user: true,
        }
    }
}
