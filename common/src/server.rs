use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::fs::DirTree;

#[async_trait]
pub trait Server {
    async fn init(&self);
    async fn quit(&self);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerStatus {
    Connected,
    NotConnected,
}

#[derive(Debug, Clone)]
pub struct ClientInfo {
    user_id: String,
    socket: SocketAddr,
    dir_tree: DirTree,
    hostname: String,
}
