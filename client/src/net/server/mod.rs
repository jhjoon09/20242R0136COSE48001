use std::net::Ipv4Addr;
use std::sync::Arc;

use kudrive_common::ServerMessage;
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};

mod listener;
use listener::Listener;

pub struct Server {
    stream: Option<Arc<Mutex<TcpStream>>>,
    listener: Option<Listener>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            stream: None,
            listener: None,
        }
    }

    pub async fn connect(&mut self) -> io::Result<()> {
        // TODO: server address configuration
        let address = format!("{}:{}", Ipv4Addr::LOCALHOST, 7878);
        let stream = TcpStream::connect(address).await?;
        self.stream = Some(Arc::new(Mutex::new(stream)));
        self.spawn().await?;
        Ok(())
    }

    pub async fn disconnect(&mut self) -> io::Result<()> {
        if let Some(stream) = self.stream.take() {
            let mut lock = stream.lock().await;
            lock.shutdown().await?;
        }
        Ok(())
    }

    pub async fn send(&mut self, data: &[u8]) -> io::Result<()> {
        if let Some(stream) = &self.stream {
            let mut lock = stream.lock().await;
            lock.write_all(data).await?;
            drop(lock);
        }
        Ok(())
    }

    pub async fn spawn(&mut self) -> io::Result<()> {
        let (sender, mut receiver) = mpsc::channel::<ServerMessage>(1024);
        let stream = self.stream.clone().unwrap();

        self.listener = Some(Listener::spawn(stream, sender));

        // TODO: handle received messages
        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                println!("Received: {:?}", message);
            }
        });

        Ok(())
    }
}
