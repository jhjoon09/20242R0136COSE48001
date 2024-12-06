mod meta;

use kudrive_common::{event::Event, message::client::ClientMessage};
pub use meta::MetaEvent;

#[derive(Debug, Clone)]
pub enum ServerEvent {
    Message { message: ClientMessage },
    // TODO: other events
}

impl Event<ClientMessage> for ServerEvent {
    fn from_message(message: ClientMessage) -> Self {
        ServerEvent::Message { message }
    }
}
