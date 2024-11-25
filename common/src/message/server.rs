use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    HealthCheck { timestamp: SystemTime },
}

impl ServerMessage {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        match serde_json::from_slice(bytes) {
            Ok(message) => message,
            Err(e) => {
                eprintln!("Failed to parse JSON data: {:?}", e);
                ServerMessage::HealthCheck {
                    timestamp: SystemTime::now(),
                }
            }
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match serde_json::to_vec(self) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Failed to serialize JSON data: {:?}", e);
                Vec::new()
            }
        }
    }
}
