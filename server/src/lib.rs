pub mod client;
pub mod event;

use event::MetaEvent;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
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
        self.connect(handler.clone()).await;

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

    async fn connect(&mut self, handler: Arc<Mutex<ClientHandler>>) {
        self.clients.entry(None).or_insert(vec![]).push(handler);
    }

    async fn register(&mut self, group: Uuid, id: Uuid) {
        // TODO: implement client registration
    }

    async fn propagate(&mut self, group: Uuid) {
        // TODO: implement client propagation
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
                            MetaEvent::Register { group, id } => {
                                self.register(group, id).await;
                            }
                            MetaEvent::Propagation { group } => {
                                self.propagate(group).await;
                        }
                    }
                }
            }
        }
    }
}
