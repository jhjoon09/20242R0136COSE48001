use crate::{
    file_server::FileServer,
    net::{p2p::P2PTransport, server::Server},
};
use kudrive_common::{
    event::{
        client::{
            command::{Command, Consequence},
            ClientEvent, ServerMessage,
        },
        server::ClientMessage,
    },
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
    clients: (),
}

impl Client {
    pub fn new() -> Self {
        // create event channel
        let channel = mpsc::channel::<ClientEvent>(1024);
        let (sender, receiver) = channel;

        Self {
            server: Server::new(),
            file_server: FileServer::new(sender.clone()),
            p2p_transport: P2PTransport::new(sender.clone()),
            sender,
            receiver,
            pendings: Pendings::new(),
            clients: (),
        }
    }

    pub fn sender(&self) -> Sender<ClientEvent> {
        self.sender.clone()
    }

    fn try_receive(&mut self) -> Result<ClientEvent, TryRecvError> {
        self.receiver.try_recv()
    }

    fn set_clients(&mut self, clients: ()) {
        self.clients = clients;
    }

    async fn get_clients(&self, id: u64) {
        let clients = self.clients;
        let responder = self.sender();

        tokio::spawn(async move {
            responder
                .send(ClientEvent::Consequence {
                    id,
                    consequence: Consequence::Clients {
                        result: Ok(clients),
                    },
                })
                .await
                .unwrap();
        });
    }

    pub async fn start(&mut self) {
        if let Err(e) = self.server.connect(self.sender()).await {
            eprintln!("Failed to connect to server: {:?}", e);
            return;
        }

        self.file_server.start().await;
        self.p2p_transport.connect().await;

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
                match message {
                    ServerMessage::HealthCheck { timestamp } => {
                        println!("Health check: {:?}", timestamp);
                    }
                    ServerMessage::ClientsUpdate { clients } => {
                        // TODO: implement clients update
                        self.set_clients(clients);
                    }
                }
            }
            ClientEvent::FileMapUpdate { file_map } => {
                // TODO: implement file map update
                let message = ClientMessage::FileMapUpdate { file_map };
                self.server.transmit(message).await.unwrap();
            }
            ClientEvent::Command { command, responder } => {
                println!("Received command: {:?}", command);
                let id = self.pendings.insert(responder);
                match command {
                    Command::FileSend { target, from, to } => {
                        self.p2p_transport.send_file(id, target, from, to).await;
                    }
                    Command::FileReceive { target, from, to } => {
                        self.p2p_transport.receive_file(id, target, from, to).await;
                    }
                    Command::Clients {} => {
                        self.get_clients(id).await;
                    }
                }
            }
            ClientEvent::Consequence { id, consequence } => {
                if let Some(responder) = self.pendings.remove(id) {
                    responder.send(consequence).unwrap();
                }
            } // TODO: implement clients broadcast message
              // TODO: implement health check message
        };
        Ok(())
    }

    pub async fn shutdown(&mut self) {
        let _ = self.server.disconnect().await;
        self.file_server.stop().await;

        println!("Client shutdown.");
    }
}
