pub mod client;

use kudrive_common::event::server::MetaEvent;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};
use uuid::Uuid;

use client::handler::ClientHandler;

pub struct Server {
    clients: HashMap<Option<Uuid>, Vec<Arc<Mutex<ClientHandler>>>>,
}

impl Server {
    pub async fn new() -> Self {
        Self {
            clients: HashMap::new(),
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
                        _ => {}
                    }
                }
            }
        }
    }
}
