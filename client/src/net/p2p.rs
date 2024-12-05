use uuid::Uuid;

pub struct P2PTransport {}

impl P2PTransport {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn connect(&self) {}

    pub async fn disconnect(&self) {}

    pub async fn send_open(&self, source: String) -> Result<(), String> {
        println!("Opening for sending file: {}", source);
        Ok(())
    }

    pub async fn send_wait(&self, source: String) -> Result<(), String> {
        println!("Waiting for sending file: {}", source);
        Ok(())
    }

    pub async fn receive(&self, peer: Uuid, source: String, target: String) -> Result<(), String> {
        println!("Receiving file from {}: {} -> {}", peer, source, target);
        Ok(())
    }
}
