use kudrive_common::ClientInfo;
use std::net::Ipv4Addr;

pub struct Peer {
    client: ClientInfo,
    addr: Ipv4Addr,
}

pub struct P2PTransport {
    peers: Vec<Peer>,
}

impl P2PTransport {
    pub fn new() -> Self {
        Self { peers: Vec::new() }
    }

    pub async fn connect(&self) {
        println!("Connecting to peers...");
    }

    pub async fn disconnect(&self) {
        println!("Disconnecting from peers...");
    }

    pub async fn broadcast(&self, data: Vec<u8>) {
        println!("Broadcasting data to peers...");
    }
}
