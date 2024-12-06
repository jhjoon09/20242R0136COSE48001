use std::collections::HashMap;

use kudrive_common::Client;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::event::ServerEvent;

#[derive(Debug, Clone)]
pub struct ClientGroup {
    clients: HashMap<Uuid, Client>,
    senders: HashMap<Uuid, Sender<ServerEvent>>,
}

impl ClientGroup {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            senders: HashMap::new(),
        }
    }

    pub fn insert(&mut self, client: Client, sender: Sender<ServerEvent>) {
        let Client { id, .. } = client;
        self.clients.insert(id, client);
        self.senders.insert(id, sender);
    }

    pub fn update(&mut self, client: Client) {
        let Client { id, .. } = client;
        self.clients.insert(id, client);
    }

    pub fn remove(&mut self, id: Uuid) {
        self.clients.remove(&id);
        self.senders.remove(&id);
    }

    pub fn flatten(&self) -> Vec<Client> {
        self.clients.values().cloned().collect()
    }

    pub fn find_by_nickname(&self, nickname: &str) -> Option<Uuid> {
        self.clients
            .values()
            .find(|client| client.nickname == nickname)
            .map(|client| {
                let Client { id, .. } = client;
                *id
            })
    }
}
