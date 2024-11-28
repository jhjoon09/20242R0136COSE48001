use kudrive_common::event::client::{command::Consequence, ClientEvent};
use tokio::sync::mpsc::Sender;

pub struct FileServer;

impl FileServer {
    pub fn new() -> Self {
        Self
    }

    pub async fn start(&self) {
        println!("File server started.");
    }

    pub async fn stop(&self) {
        println!("File server stopped.");
    }

    pub async fn send_file(
        &self,
        responder: Sender<ClientEvent>,
        id: u64,
        target: String,
        from: String,
        to: String,
    ) {
        tokio::spawn(async move {
            /* TODO: logics for send file */
            println!("Sending file: from my {} to cilent {} {}", from, target, to);

            responder
                .send(ClientEvent::Consequence {
                    id,
                    consequence: Consequence::FileSend { result: Ok(()) },
                })
                .await
        });
    }

    pub async fn receive_file(
        &self,
        responder: Sender<ClientEvent>,
        id: u64,
        target: String,
        from: String,
        to: String,
    ) {
        tokio::spawn(async move {
            /* TODO: logics for receive file */
            println!("Receiving file: from {} {} to my {}", target, from, to);

            responder
                .send(ClientEvent::Consequence {
                    id,
                    consequence: Consequence::FileReceive { result: Ok(()) },
                })
                .await
        });
    }
}
