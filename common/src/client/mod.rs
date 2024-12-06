use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::FileMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub group: Uuid,
    pub id: Uuid,
    pub nickname: String,
    pub files: FileMap,
}
