use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::FileMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    group: Uuid,
    id: Uuid,
    nickname: String,
    files: FileMap,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Peer {
    // TODO: Add more fields
}
