use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileClaim {
    SendClaim {},
    ReceiveClaim {},
    WaitClaim {},
}
