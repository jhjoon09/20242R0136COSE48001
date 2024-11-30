use std::sync::Arc;

use kudrive_common::Client;
use tokio::{net::TcpStream, sync::Mutex};

pub struct ClientHandler {
    stream: Arc<Mutex<TcpStream>>,
    info: Option<Client>,
}

impl ClientHandler {
    pub async fn new(stream: TcpStream) -> Self {
        let stream = Arc::new(Mutex::new(stream));

        Self { stream, info: None }
    }

    pub async fn start(&self) {
        tokio::spawn(async move {
            todo!();
        });
    }
}
