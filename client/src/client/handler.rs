use crate::{
    file_server::FileServer,
    net::{p2p::P2PTransport, server::Server},
    p2p,
};
use dotenv::{dotenv, from_path};
use kudrive_common::{
    event::{
        client::{
            command::{Command, Consequence},
            ClientEvent, ServerMessage,
        },
        server::ClientMessage,
    },
    util::Pendings,
    Client,
};
use tokio::sync::{
    mpsc::{self, error::TryRecvError, Receiver, Sender},
    oneshot,
};

pub struct ClientHandler {
    sender: Sender<ClientEvent>,
    receiver: Receiver<ClientEvent>,
    pub file_server: FileServer,
    pub server: Server,
    pub p2p_transport: P2PTransport,
    pendings: Pendings<oneshot::Sender<Consequence>>,
    clients: Vec<Client>,
}

impl ClientHandler {
    pub fn new() -> Self {
        // let _ = from_path(".client.env");
        // dotenv().ok();
        // let client_hostname = env::var("CLIENT_NAME").expect("CLIENT_NAME must be set");
        // let relay_address = env::var("RELAY_ADDR").expect("RELAY_ADDR must be set");
        // let p2p_transport = P2PTransport::new(&relay_address, &client_hostname)
        //     .await
        //     .expect("Failed to create P2P client");
        let p2p_transport = P2PTransport::new_mock();

        // create event channel
        let channel = mpsc::channel::<ClientEvent>(1024);
        let (sender, receiver) = channel;

        Self {
            server: Server::new(),
            file_server: FileServer::new(sender.clone()),
            p2p_transport,
            sender,
            receiver,
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
        self.server.transmit(message).await.unwrap();
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

    pub async fn start(&mut self) {
        if let Err(e) = self.server.connect(self.sender()).await {
            eprintln!("Failed to connect to server: {:?}", e);
            return;
        }

        self.file_server.start().await;
        // self.p2p_transport.connect_relay(10).await;
        // self.p2p_transport.listen_on_peer(10).await;

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
                self.transmit(message).await;
            }
            ClientEvent::Command { command, responder } => {
                println!("Received command: {:?}", command);
                let id = self.pendings.insert(responder);
                match command {
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
