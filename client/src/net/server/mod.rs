use std::net::Ipv4Addr;
use std::sync::Arc;

use kudrive_common::event::client::{ClientEvent, ServerMessage};
use kudrive_common::message::client::ClientMessage;
use kudrive_common::{Listener, Transmitter};
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

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
        // TODO: server address configuration
        let address = format!("{}:{}", Ipv4Addr::LOCALHOST, 7878);

        // create tcp stream
        let stream = TcpStream::connect(address).await?;
        self.stream = Some(Arc::new(Mutex::new(stream)));

        // create transmitter and listener
        self.transmitter = Some(Transmitter::new(self.clone_stream()));
        self.listener = Some(Listener::spawn(self.clone_stream(), sender));

        Ok(())
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
