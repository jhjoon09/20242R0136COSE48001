use std::time::Duration;

use crate::{
    event::{ClientEvent, Command, Consequence},
    file_server::FileServer,
    net::{p2p::P2PTransport, server::Server},
};
use kudrive_common::{
    health::HealthChecker,
    message::{client::ClientMessage, server::ServerMessage, FileClaim},
    pending::Pendings,
    Client,
};
use tokio::sync::{
    mpsc::{self, error::TryRecvError, Receiver, Sender},
    oneshot,
};

pub struct ClientHandler {
    sender: Sender<ClientEvent>,
    receiver: Receiver<ClientEvent>,
    health_checker: Option<HealthChecker<ServerMessage, ClientEvent>>,
    file_server: FileServer,
    pub server: Server,
    pub p2p_transport: P2PTransport,
    pendings: Pendings<oneshot::Sender<Consequence>>,
    clients: Vec<Client>,
}

impl ClientHandler {
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
            health_checker: None,
            pendings: Pendings::new(),
            clients: Vec::new(),
        }
    }

    pub fn sender(&self) -> Sender<ClientEvent> {
        self.sender.clone()
    }

    fn try_receive(&mut self) -> Result<ClientEvent, TryRecvError> {
        self.receiver.try_recv()
    }

    fn set_clients(&mut self, clients: Vec<Client>) {
        self.clients = clients;
    }

    async fn transmit(&mut self, message: ClientMessage) {
        if let Err(_) = self.server.transmit(message).await {
            self.sender.send(ClientEvent::Unhealthy {}).await.unwrap();
        }
    }

    async fn send_event(&self, event: ClientEvent) {
        self.sender.send(event).await.unwrap();
    }

    async fn get_clients(&self, id: u64) {
        let clients = self.clients.clone();
        let consequence = Consequence::Clients {
            result: Ok(clients),
        };

        let event = ClientEvent::Consequence { id, consequence };
        self.send_event(event).await;
    }

    async fn connect_server(&mut self) {
        // connect to server
        loop {
            match self.server.connect(self.sender()).await {
                Ok(_) => break,
                Err(e) => {
                    eprintln!("Failed to connect to server: {:?}", e);
                }
            }
        }

        // register to server
        loop {
            match self.server.register().await {
                Ok(_) => break,
                Err(e) => {
                    eprintln!("Failed to register to server: {:?}", e);
                }
            }
        }

        // spawn health checker
        let health_checker = HealthChecker::new(
            self.sender.clone(),
            ClientEvent::Unhealthy {},
            Duration::from_secs(5),
        );
        health_checker.check().await;
        self.health_checker = Some(health_checker);
    }

    pub async fn start(&mut self) {
        self.connect_server().await;

        self.file_server.start().await;
        self.p2p_transport.connect().await;

        // spawn health check send timer
        let sender = self.sender.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                sender.send(ClientEvent::Timer {}).await.unwrap();
            }
        });

        println!("Client started.");
    }

    pub async fn event_listen(&mut self) -> Result<(), TryRecvError> {
        let event = match self.try_receive() {
            Ok(event) => event,
            Err(TryRecvError::Empty) => return Ok(()),
            Err(e) => return Err(e),
        };

        match event {
            ClientEvent::Message { message } => match message {
                ServerMessage::HealthCheck {} => match self.health_checker {
                    Some(ref mut health_checker) => health_checker.check().await,
                    None => self.sender().send(ClientEvent::Unhealthy {}).await.unwrap(),
                },
                ServerMessage::ClientsUpdate { clients } => {
                    self.set_clients(clients);
                }
                ServerMessage::FileClaim { claim, peer } => match claim {
                    FileClaim::SendClaim { pending } => {
                        self.p2p_transport.send_open(false, pending, peer).await;
                    }
                    FileClaim::ReceiveClaim { pending } => {
                        self.p2p_transport.receive(pending, peer).await;
                    }
                },
            },
            ClientEvent::FileMapUpdate { file_map } => {
                // TODO: implement file map update
                let message = ClientMessage::FileMapUpdate { file_map };
                self.transmit(message).await;
            }
            ClientEvent::Command { command, responder } => {
                println!("Received command: {:?}", command);
                let id = self.pendings.insert(responder);
                match command {
                    Command::Clients {} => {
                        self.get_clients(id).await;
                    }
                    Command::FileSend { peer } => {
                        self.p2p_transport.send_open(true, id, peer).await;
                    }
                    Command::FileReceive { peer } => {
                        let message = ClientMessage::FileClaim {
                            claim: FileClaim::SendClaim { pending: id },
                            peer,
                        };
                        println!("Sending file claim: {:?}", message);
                        self.transmit(message).await;
                    }
                }
            }
            ClientEvent::Consequence { id, consequence } => {
                if let Some(responder) = self.pendings.remove(id) {
                    responder.send(consequence).unwrap();
                }
            }
            ClientEvent::Opened { ids, convey } => {
                let (wid, rid) = ids;
                let (peer, rx) = convey;

                let message = ClientMessage::FileClaim {
                    claim: FileClaim::ReceiveClaim { pending: rid },
                    peer: peer.clone(),
                };
                self.transmit(message).await;

                self.p2p_transport.send_wait(wid, peer, rx).await;
            }
            ClientEvent::Timer {} => {
                self.transmit(ClientMessage::HealthCheck {}).await;
            }
            ClientEvent::Unhealthy {} => {
                println!("Server is unhealthy.");
                self.connect_server().await;
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
