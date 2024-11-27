use tokio::sync::oneshot;

use super::Event;
pub use crate::message::server::ServerMessage;

#[derive(Debug)]

pub enum Command {
    FileSend {
        target: String,
        from: String,
        to: String,
    },
    FileReceive {
        target: String,
        from: String,
        to: String,
    },
    Clients {},
}

#[derive(Debug)]
pub enum Consequence {
    FileSend { result: Result<(), String> },
    FileReceive { result: Result<(), String> },
    Clients { result: Result<(), String> },
}

#[derive(Debug)]
pub enum ClientEvent {
    Message {
        message: ServerMessage,
    },
    Command {
        command: Command,
        responder: oneshot::Sender<Consequence>,
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
