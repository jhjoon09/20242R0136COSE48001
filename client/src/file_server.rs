use kudrive_common::event::client::ClientEvent;
use tokio::sync::mpsc::Sender;

pub struct FileServer;

impl FileServer {
    pub fn new() -> Self {
        Self
    }

    pub async fn start(&self) {
        println!("File server started.");
    }

    /* TODO: real send file */
    pub async fn send_file(
        &self,
        sender: Sender<ClientEvent>,
        id: u64,
        target: String,
        from: String,
        to: String,
    ) {
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            sender
                .send(ClientEvent::Consequence {
                    id,
                    consequence: kudrive_common::event::client::Consequence::FileSend {
                        result: Ok(()),
                    },
                })
                .await
        });
        println!("Sending file to {} from {} to {}", target, from, to);
    }

    pub async fn stop(&self) {
        println!("File server stopped.");
    }
}
