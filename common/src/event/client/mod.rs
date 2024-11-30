pub mod command;

pub use command::{Command, Consequence};
use tokio::sync::oneshot;

use super::Event;
pub use crate::message::server::ServerMessage;

#[derive(Debug)]
pub enum ClientEvent {
    Message {
        message: ServerMessage,
    },
    Command {
        command: Command,
        responder: oneshot::Sender<Consequence>,
    },
    FileMapUpdate {
        file_map: (),
    },
    Consequence {
        id: u64,
        consequence: Consequence,
    },
}

impl Event<ServerMessage> for ClientEvent {
    fn from_message(message: ServerMessage) -> Self {
        ClientEvent::Message { message }
    }
}
