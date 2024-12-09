use tokio::sync::mpsc::Sender;

use crate::Client;

use super::ServerEvent;

pub enum MetaEvent {
    Register {
        client: Client,
        sender: Sender<ServerEvent>,
    },
}
