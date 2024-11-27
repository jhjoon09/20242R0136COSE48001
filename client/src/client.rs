use crate::{
    file_server::FileServer,
    net::{p2p::P2PTransport, server::Server},
};
use kudrive_common::{
    event::client::{ClientEvent, Command, Consequence},
    util::Pendings,
};
use tokio::sync::{
    mpsc::{self, error::TryRecvError, Receiver, Sender},
    oneshot,
};

pub struct Client {
    sender: Sender<ClientEvent>,
    receiver: Receiver<ClientEvent>,
    pub file_server: FileServer,
    pub server: Server,
    pub p2p_transport: P2PTransport,
    pendings: Pendings<oneshot::Sender<Consequence>>,
}

impl Client {
    pub fn new() -> Self {
        // create event channel
        let channel = mpsc::channel::<ClientEvent>(1024);
        let (sender, receiver) = channel;

        Self {
            sender,
            receiver,
            file_server: FileServer::new(),
            server: Server::new(),
            p2p_transport: P2PTransport::new(),
            pendings: Pendings::new(),
        }
    }

    pub fn sender(&self) -> Sender<ClientEvent> {
        self.sender.clone()
    }

    fn try_receive(&mut self) -> Result<ClientEvent, TryRecvError> {
        self.receiver.try_recv()
    }

    pub async fn start(&mut self) {
        if let Err(e) = self.server.connect(self.sender()).await {
            eprintln!("Failed to connect to server: {:?}", e);
            return;
        }

        // self.file_server.start().await;
        // self.p2p_transport.connect().await;

        println!("Client started.");
    }

    pub async fn event_listen(&mut self) -> Result<(), TryRecvError> {
        let event = match self.try_receive() {
            Ok(event) => event,
            Err(TryRecvError::Empty) => return Ok(()),
            Err(e) => return Err(e),
        };

        match event {
            ClientEvent::Message { message } => {
                println!("Received message: {:?}", message);
            }
            ClientEvent::Command { command, responder } => {
                println!("Received command: {:?}", command);
                let sender = self.sender();
                let id = self.pendings.insert(responder);
                match command {
                    Command::FileSend { target, from, to } => {
                        self.file_server
                            .send_file(sender, id, target, from, to)
                            .await;
                    }
                    _ => {}
                }
            }
            ClientEvent::Consequence { id, consequence } => {
                if let Some(responder) = self.pendings.remove(id) {
                    let _ = responder.send(consequence);
                }
            }
        };
        Ok(())
    }

    pub async fn shutdown(&mut self) {
        let _ = self.server.disconnect().await;
        self.file_server.stop().await;

        println!("Client shutdown.");
    }
}
