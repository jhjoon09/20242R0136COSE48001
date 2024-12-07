use serde::{Deserialize, Serialize};
use serde_json;

use crate::Client;

use super::Message;

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    HealthCheck {},
    ClientsUpdate { clients: Vec<Client> },
}

impl Message for ServerMessage {
    fn from_bytes(bytes: &[u8]) -> Self {
        match serde_json::from_slice(bytes) {
            Ok(message) => message,
            Err(e) => {
                eprintln!("Failed to parse JSON data: {:?}", e);
                ServerMessage::HealthCheck {}
            }
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        match serde_json::to_vec(self) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Failed to serialize JSON data: {:?}", e);
                Vec::new()
            }
        }
    }
}
