// lib.rs
pub mod file_server;
pub mod net;

use file_server::FileServer;
use net::p2p::P2PTransport;
use net::server::Server;

pub struct Client {
    pub file_server: FileServer,
    pub server: Server,
    pub p2p_transport: P2PTransport,
}

impl Client {
    pub fn new() -> Self {
        Self {
            file_server: FileServer::new(),
            server: Server::new(),
            p2p_transport: P2PTransport::new(),
        }
    }

    pub async fn start(&self) {
        self.file_server.start().await;
        self.server.connect().await;
        self.p2p_transport.connect().await;
        println!("Client started.");
    }

    pub async fn shutdown(&self) {
        self.server.disconnect().await;
        self.file_server.stop().await;
        println!("Client shutdown.");
    }
}
