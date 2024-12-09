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

    fn send(&self, sender: &Sender<ServerEvent>, event: &ServerEvent) {
        let sender = sender.clone();
        let event = event.clone();

        tokio::spawn(async move {
            let _ = sender.send(event).await;
        });
    }

    pub async fn unicast(&self, id: Uuid, event: ServerEvent) {
        if let Some(sender) = self.senders.get(&id) {
            self.send(sender, &event);
        }
    }

    pub async fn broadcast(&self, event: ServerEvent) {
        for sender in self.senders.values() {
            self.send(sender, &event);
        }
    }
}
