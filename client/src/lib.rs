// lib.rs
pub mod file_server;
pub mod net;

use file_server::FileServer;
use kudrive_common::event::client::ClientEvent;
use net::p2p::P2PTransport;
use net::server::Server;
use tokio::sync::mpsc::{self, Receiver, Sender};

pub struct Client {
    sender: Sender<ClientEvent>,
    receiver: Option<Receiver<ClientEvent>>,
    pub file_server: FileServer,
    pub server: Server,
    pub p2p_transport: P2PTransport,
}

impl Client {
    pub fn new() -> Self {
        // create event channel
        let channel = mpsc::channel::<ClientEvent>(1024);
        let (sender, receiver) = channel;

        Self {
            sender,
            receiver: Some(receiver),
            file_server: FileServer::new(),
            server: Server::new(),
            p2p_transport: P2PTransport::new(),
        }
    }

    fn sender(&self) -> Sender<ClientEvent> {
        self.sender.clone()
    }

    fn receiver(&mut self) -> Option<Receiver<ClientEvent>> {
        self.receiver.take()
    }

    pub async fn start(&mut self) {
        if let Err(e) = self.server.connect(self.sender()).await {
            eprintln!("Failed to connect to server: {:?}", e);
            return;
        }

        // self.file_server.start().await;
        // self.p2p_transport.connect().await;

        println!("Client started.");

        self.event_listen().await;
    }

    async fn event_listen(&mut self) {
        let mut receiver = self.receiver().unwrap();

        // event loop
        while let Some(message) = receiver.recv().await {
            match message {
                ClientEvent::Message { message } => {
                    println!("Received message: {:?}", message);
                }
            }
        }
    }

    pub async fn shutdown(&mut self) {
        let _ = self.server.disconnect().await;
        self.file_server.stop().await;

        println!("Client shutdown.");
    }
}
