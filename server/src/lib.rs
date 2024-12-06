pub mod client;
pub mod event;

use event::{MetaEvent, PeerEvent, ServerEvent};
use kudrive_common::Client;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{self},
        Mutex, RwLock,
    },
};
use uuid::Uuid;

pub use client::{group::ClientGroup, handler::ClientHandler};

pub struct Server {
    groups: HashMap<Uuid, Arc<RwLock<ClientGroup>>>,
}

impl Server {
    pub async fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    async fn spawn(&mut self, stream: TcpStream, sender: mpsc::Sender<MetaEvent>) {
        let handler = Arc::new(Mutex::new(ClientHandler::new(stream, sender)));

        tokio::spawn(async move {
            let handler = handler.clone();
            loop {
                let mut handler = handler.lock().await;
                if let Err(_) = handler.event_listen().await {
                    // TODO: handle client error
                }
                drop(handler);
            }
        });
    }

    async fn register(&mut self, client: Client, sender: mpsc::Sender<ServerEvent>) {
        // get target group
        let Client { group, .. } = client;
        let group = self
            .groups
            .entry(group)
            .or_insert_with(|| Arc::new(RwLock::new(ClientGroup::new())));

        // send client its group
        let event = ServerEvent::PeerEvent {
            event: PeerEvent::Group {
                group: group.clone(),
            },
        };
        sender.send(event).await.unwrap();

        // add client to group
        let mut lock = group.write().await;
        lock.insert(client, sender);
        drop(lock);

        // broadcast client register
        let event = ServerEvent::PeerEvent {
            event: PeerEvent::Update {},
        };
        let lock = group.write().await;
        lock.broadcast(event).await;
        drop(lock);
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("127.0.0.1:7878").await?;
        let channel = mpsc::channel::<MetaEvent>(1024);

        let (sender, mut receiver) = channel;

        loop {
            tokio::select! {
                Ok((stream, addr)) = listener.accept() => {
                    println!("Connection from: {}", addr);
                    self.spawn(stream, sender.clone()).await;
                }
                Some(event) = receiver.recv() => {
                    match event {
                        MetaEvent::Register { client, sender } => {
                            self.register(client, sender).await;
                        }
                    }
                }
            }
        }
    }
}
