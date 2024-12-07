use std::{sync::Arc, time::Duration};

use kudrive_common::{
    health::HealthChecker,
    message::{client::ClientMessage, server::ServerMessage},
    tcp::transmitter,
    Client, FileMap, Listener, Transmitter,
};
use tokio::{
    net::TcpStream,
    sync::{
        mpsc::{self, error::TryRecvError, Receiver, Sender},
        Mutex, RwLock,
    },
};

use crate::event::{MetaEvent, PeerEvent, ServerEvent};

use super::group::ClientGroup;

pub struct ClientHandler {
    client: Option<Client>,
    group: Option<Arc<RwLock<ClientGroup>>>,
    meta: Sender<MetaEvent>,
    sender: Sender<ServerEvent>,
    receiver: Receiver<ServerEvent>,
    transmitter: Transmitter,
    health_checker: HealthChecker<ClientMessage, ServerEvent>,
}

impl ClientHandler {
    pub fn new(stream: TcpStream, meta: mpsc::Sender<MetaEvent>) -> Self {
        let stream = Arc::new(Mutex::new(stream));
        let (sender, receiver) = mpsc::channel::<ServerEvent>(1024);

        let transmitter = transmitter::Transmitter::new(stream.clone());
        let _listener = Listener::spawn(stream.clone(), sender.clone());

        let health_checker = HealthChecker::new(
            sender.clone(),
            ServerEvent::Unhealthy {},
            Duration::from_secs(5),
        );

        Self {
            client: None,
            group: None,
            meta,
            sender,
            receiver,
            transmitter,
            health_checker,
        }
    }

    fn sender(&self) -> Sender<ServerEvent> {
        self.sender.clone()
    }

    async fn register(&mut self, client: Client) {
        self.client = Some(client.clone());

        let event = MetaEvent::Register {
            client: client,
            sender: self.sender(),
        };

        self.meta.send(event).await.unwrap();
    }

    async fn update(&mut self, file_map: FileMap) {
        if let Some(client) = &self.client {
            let client = Client {
                files: file_map,
                ..client.clone()
            };
            self.client = Some(client.clone());

            if let Some(group) = &self.group {
                let event = ServerEvent::PeerEvent {
                    event: PeerEvent::Update {},
                };

                let mut lock = group.write().await;
                lock.update(client);
                lock.broadcast(event).await;
                drop(lock);
            }
        }
    }

    async fn remove(&mut self) {
        if let Some(client) = &self.client {
            if let Some(group) = &self.group {
                let event = ServerEvent::PeerEvent {
                    event: PeerEvent::Update {},
                };

                let mut lock = group.write().await;
                lock.remove(client.id);
                lock.broadcast(event).await;
                drop(lock);
            }
        }
    }

    async fn transmit(&mut self, message: ServerMessage) {
        self.transmitter.send(message).await.unwrap();
    }

    async fn propagate(&mut self) {
        if let Some(group) = &self.group {
            let lock = group.read().await;
            let clients = lock.flatten();
            drop(lock);

            let message = ServerMessage::ClientsUpdate { clients };
            self.transmit(message).await;
        }
    }

    fn try_receive(&mut self) -> Result<ServerEvent, TryRecvError> {
        self.receiver.try_recv()
    }

    pub async fn event_listen(&mut self) -> Result<(), TryRecvError> {
        let event = match self.try_receive() {
            Ok(event) => event,
            Err(TryRecvError::Empty) => return Ok(()),
            Err(e) => return Err(e),
        };

        match event {
            ServerEvent::Message { message } => match message {
                ClientMessage::HealthCheck {} => {
                    self.health_checker.check().await;
                    self.transmit(ServerMessage::HealthCheck {}).await;
                }
                ClientMessage::Register { client } => {
                    println!("Registering client: {:?}", client);
                    self.register(client).await;
                }
                ClientMessage::FileMapUpdate { file_map } => {
                    println!("Updating file map: {:?}", file_map);
                    self.update(file_map).await;
                }
            },
            ServerEvent::PeerEvent { event } => match event {
                PeerEvent::Update {} => {
                    println!("Propagating file map update");
                    self.propagate().await;
                }
                PeerEvent::Group { group } => {
                    self.group = Some(group);
                }
            },
            ServerEvent::Unhealthy {} => {
                self.remove().await;
            }
        };

        Ok(())
    }
}
