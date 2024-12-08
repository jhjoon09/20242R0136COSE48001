use crate::{Client, Peer};

use super::{super::super::fs::FileMap, FileClaim};
use serde::{Deserialize, Serialize};
use serde_json;

use super::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    HealthCheck {},
    Register { client: Client },
    FileMapUpdate { file_map: FileMap },
    FileClaim { claim: FileClaim, peer: Peer },
}

impl Message for ClientMessage {
    fn from_bytes(bytes: &[u8]) -> Self {
        match serde_json::from_slice(bytes) {
            Ok(message) => message,
            Err(e) => {
                eprintln!("Failed to parse JSON data: {:?}", e);
                ClientMessage::HealthCheck {}
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
