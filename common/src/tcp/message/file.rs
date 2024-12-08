use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileClaim {
    SendClaim { pending: u64 },
    ReceiveClaim { pending: Option<u64> },
}
