use tokio::{sync::oneshot, time};

use kudrive_common::Peer;
use tokio::sync::mpsc::Sender;

use crate::event::{ClientEvent, Consequence};

pub struct P2PTransport {
    responder: Sender<ClientEvent>,
}

impl P2PTransport {
    pub fn new(responder: Sender<ClientEvent>) -> Self {
        Self { responder }
    }

    fn responder(&self) -> Sender<ClientEvent> {
        self.responder.clone()
    }

    pub async fn connect(&self) {}

    pub async fn disconnect(&self) {}

    pub async fn send_open(&self, own: bool, pending: u64, peer: Peer) {
        let responder = self.responder();

        tokio::spawn(async move {
            /* START TEMP */
            println!("Opening for sending file: {}", peer.source);
            time::sleep(time::Duration::from_secs(5)).await;
            println!("Opened for sending file: {}", peer.source);
            let (tx, rx) = oneshot::channel();
            /* END TEMP */

            let ids = match own {
                true => (Some(pending), None),
                false => (None, Some(pending)),
            };

            let convey = (peer, rx);
            let event = ClientEvent::Opened { ids, convey };
            responder.send(event).await.unwrap();
        });
        ()
    }

    pub async fn send_wait(
        &self,
        pending: Option<u64>,
        peer: Peer,
        rx: oneshot::Receiver<Result<(), String>>,
    ) {
        let responder = self.responder();

        tokio::spawn(async move {
            /* START TEMP */
            println!("Waiting for sending file: {}", peer.source);
            time::sleep(time::Duration::from_secs(5)).await;
            println!("Waited for sending file: {}", peer.source);
            /* END TEMP */

            if let Some(id) = pending {
                let consequence = Consequence::FileSend { result: Ok(()) };
                let event = ClientEvent::Consequence { id, consequence };
                responder.send(event).await.unwrap();
            }
        });
        ()
    }

    pub async fn receive(&self, pending: Option<u64>, peer: Peer) {
        let responder = self.responder();

        tokio::spawn(async move {
            println!(
                "Receiving file from {}: {} -> {}",
                peer.id, peer.source, peer.target
            );
            time::sleep(time::Duration::from_secs(5)).await;
            println!(
                "Received file from {}: {} -> {}",
                peer.id, peer.source, peer.target
            );
            /* END TEMP */

            if let Some(id) = pending {
                let consequence = Consequence::FileReceive { result: Ok(()) };
                let event = ClientEvent::Consequence { id, consequence };
                responder.send(event).await.unwrap();
            }
        });
        ()
    }
}
