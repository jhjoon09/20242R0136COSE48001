use std::net::Ipv4Addr;

use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct Server {
    stream: Option<TcpStream>,
}

impl Server {
    pub fn new() -> Self {
        Self { stream: None }
    }

    pub async fn connect(&mut self) -> io::Result<()> {
        // TODO: server address configuration
        let address = format!("{}:{}", Ipv4Addr::LOCALHOST, 7878);
        let stream = TcpStream::connect(address).await?;
        self.stream = Some(stream);
        Ok(())
    }

    pub async fn disconnect(&mut self) -> io::Result<()> {
        if let Some(mut stream) = self.stream.take() {
            stream.shutdown().await?;
        }
        Ok(())
    }

    pub async fn send(&mut self, data: &[u8]) -> io::Result<()> {
        if let Some(mut stream) = self.stream.take() {
            stream.write_all(data).await?;
        }
        Ok(())
    }
}
