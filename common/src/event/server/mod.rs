pub use crate::message::client::ClientMessage;

mod meta;
pub use meta::MetaEvent;

use super::Event;

#[derive(Debug)]
pub enum ServerEvent {
    Message { message: ClientMessage },
    // TODO: other events
}

impl Event<ClientMessage> for ServerEvent {
    fn from_message(message: ClientMessage) -> Self {
        ServerEvent::Message { message }
    }
}
