use std::sync::Arc;

use kudrive_common::message::client::ClientMessage;
use kudrive_common::message::server::ServerMessage;
use kudrive_common::{Client, FileMap, Listener, Transmitter};
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use crate::config_loader::get_config;
use crate::event::ClientEvent;

pub struct Server {
    stream: Option<Arc<Mutex<TcpStream>>>,
    listener: Option<Listener<ServerMessage, ClientEvent>>,
    transmitter: Option<Transmitter>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            stream: None,
            listener: None,
            transmitter: None,
        }
    }

    pub fn clone_stream(&self) -> Arc<Mutex<TcpStream>> {
        self.stream.clone().unwrap()
    }

    pub async fn connect(&mut self, sender: Sender<ClientEvent>) -> io::Result<()> {
        let config = get_config();
        let address = format!("{}:{}", config.server.domain, config.server.port);

        // create tcp stream
        let stream = TcpStream::connect(address).await?;
        self.stream = Some(Arc::new(Mutex::new(stream)));

        // create transmitter and listener
        self.transmitter = Some(Transmitter::new(self.clone_stream()));
        self.listener = Some(Listener::spawn(self.clone_stream(), sender));

        Ok(())
    }

    pub async fn register(&mut self) -> io::Result<()> {
        let config = &get_config();
        let client = Client {
            group: config.id.group_id,
            id: config.id.my_id,
            nickname: config.id.nickname.clone(),
            files: FileMap { files: vec![], folders: vec![] },
        };

        let message = ClientMessage::Register { client };
        self.transmit(message).await
    }

    pub async fn disconnect(&mut self) -> io::Result<()> {
        if let Some(stream) = self.stream.take() {
            let mut lock = stream.lock().await;
            lock.shutdown().await?;
        }
        Ok(())
    }

    pub async fn transmit(&mut self, data: ClientMessage) -> io::Result<()> {
        if let Some(transmitter) = &self.transmitter {
            transmitter.send(data).await?;
        }
        Ok(())
    }
}
