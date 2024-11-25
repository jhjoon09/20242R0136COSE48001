use std::net::Ipv4Addr;
use std::sync::Arc;

use kudrive_common::{ClientMessage, Listener, ServerMessage, Transmitter};
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};

pub struct Server {
    stream: Option<Arc<Mutex<TcpStream>>>,
    listener: Option<Listener<ServerMessage>>,
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

    pub async fn connect(&mut self) -> io::Result<()> {
        // TODO: server address configuration
        let address = format!("{}:{}", Ipv4Addr::LOCALHOST, 7878);

        // create TCP stream
        let stream = TcpStream::connect(address).await?;
        self.stream = Some(Arc::new(Mutex::new(stream)));

        // transmitter
        self.transmitter = Some(Transmitter::new(self.stream.clone().unwrap()));

        // listener
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

    pub async fn transmit(&mut self, data: ClientMessage) -> io::Result<()> {
        if let Some(transmitter) = &self.transmitter {
            transmitter.send(data).await?;
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
