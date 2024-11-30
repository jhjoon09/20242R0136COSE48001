use kudrive_common::event::client::ClientEvent;
use tokio::sync::mpsc::Sender;

pub struct FileServer {
    responder: Sender<ClientEvent>,
}

impl FileServer {
    pub fn new(responder: Sender<ClientEvent>) -> Self {
        Self { responder }
    }

    pub async fn start(&self) {
        println!("File server started.");
    }

    pub async fn stop(&self) {
        println!("File server stopped.");
    }

    fn responder(&self) -> Sender<ClientEvent> {
        self.responder.clone()
    }

    async fn file_map_update(&self) {
        let responder = self.responder();

        // TODO: logics for file map update
        tokio::spawn(async move {
            responder
                .send(ClientEvent::FileMapUpdate { file_map: () })
                .await
        });
    }
}
