use std::sync::Arc;

use kudrive_common::message::client::ClientMessage;
use kudrive_common::message::server::ServerMessage;
use kudrive_common::{Client, FileMap, Listener, Transmitter};
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use crate::config_loader::{get_group_id, get_nickname, get_server_address, get_uuid};
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
        let address = get_server_address();

        // create tcp stream
        let stream = TcpStream::connect(address).await?;
        self.stream = Some(Arc::new(Mutex::new(stream)));

        // create transmitter and listener
        self.transmitter = Some(Transmitter::new(self.clone_stream()));
        self.listener = Some(Listener::spawn(self.clone_stream(), sender));

        Ok(())
    }

    pub async fn register(&mut self) -> io::Result<()> {
        let client = Client {
            group: get_group_id(),
            id: get_uuid(),
            nickname: get_nickname(),
            files: FileMap {
                os: kudrive_common::fs::OS {
                    name: std::env::consts::OS.to_string(),
                },
                files: vec![],
                folders: vec![],
            },
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
