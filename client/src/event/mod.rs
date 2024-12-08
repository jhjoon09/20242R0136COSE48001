pub mod command;

pub use command::{Command, Consequence};
use kudrive_common::{event::Event, message::server::ServerMessage, FileMap, Peer};
use tokio::sync::oneshot;

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
        file_map: FileMap,
    },
    Consequence {
        id: u64,
        consequence: Consequence,
    },
    Opened {
        ids: (Option<u64>, Option<u64>),
        convey: (Peer, oneshot::Receiver<Result<(), String>>),
    },
    Timer {},
    Unhealthy {},
}

impl Event<ServerMessage> for ClientEvent {
    fn from_message(message: ServerMessage) -> Self {
        ClientEvent::Message { message }
    }
}
