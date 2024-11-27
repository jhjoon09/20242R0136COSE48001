use super::Event;
pub use crate::message::server::ServerMessage;

#[derive(Debug)]
pub enum ClientEvent {
    Message { message: ServerMessage },
    // TODO: other events
}

impl Event<ServerMessage> for ClientEvent {
    fn from_message(message: ServerMessage) -> Self {
        ClientEvent::Message { message }
    }
}
