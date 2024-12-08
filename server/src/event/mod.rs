use std::sync::Arc;

use crate::ClientGroup;

mod meta;

use kudrive_common::{
    event::Event,
    message::{client::ClientMessage, FileClaim},
    Peer,
};
pub use meta::MetaEvent;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub enum PeerEvent {
    Group { group: Arc<RwLock<ClientGroup>> },
    Update {},
    FileClaim { claim: FileClaim, peer: Peer },
}

#[derive(Debug, Clone)]
pub enum ServerEvent {
    Message { message: ClientMessage },
    PeerEvent { event: PeerEvent },
    Unhealthy {},
    // TODO: other events
}

impl Event<ClientMessage> for ServerEvent {
    fn from_message(message: ClientMessage) -> Self {
        ServerEvent::Message { message }
    }
}
