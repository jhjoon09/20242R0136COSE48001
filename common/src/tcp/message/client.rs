use std::time::SystemTime;

use crate::Client;

use super::super::super::fs::FileMap;
use serde::{Deserialize, Serialize};
use serde_json;

use super::Message;

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    HealthCheck { timestamp: SystemTime },
    Register { client: Client },
    FileMapUpdate { file_map: FileMap },
}

impl Message for ClientMessage {
    fn from_bytes(bytes: &[u8]) -> Self {
        match serde_json::from_slice(bytes) {
            Ok(message) => message,
            Err(e) => {
                eprintln!("Failed to parse JSON data: {:?}", e);
                ClientMessage::HealthCheck {
                    timestamp: SystemTime::now(),
                }
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
