use kudrive_common::{
    event::client::{ClientEvent, Consequence},
    Peer,
};
use tokio::sync::mpsc::Sender;

pub struct P2PTransport {
    peers: Vec<Peer>,
    responder: Sender<ClientEvent>,
}

impl P2PTransport {
    pub fn new(responder: Sender<ClientEvent>) -> Self {
        Self {
            peers: Vec::new(),
            responder,
        }
    }

    pub async fn connect(&self) {
        println!("Connecting to peers...");
    }

    pub async fn disconnect(&self) {
        println!("Disconnecting from peers...");
    }

    pub async fn broadcast(&self, data: Vec<u8>) {
        println!("Broadcasting data to peers...");
    }

    fn responder(&self) -> Sender<ClientEvent> {
        self.responder.clone()
    }

    pub async fn send_file(&self, id: u64, peer: Peer, from: String, to: String) {
        let responder = self.responder();

        tokio::spawn(async move {
            /* TODO: logics for send file */
            println!("Sending file: from my {} to cilent {:?} {}", from, peer, to);

            responder
                .send(ClientEvent::Consequence {
                    id,
                    consequence: Consequence::FileSend { result: Ok(()) },
                })
                .await
        });
    }

    pub async fn receive_file(&self, id: u64, peer: Peer, from: String, to: String) {
        let responder = self.responder();

        tokio::spawn(async move {
            /* TODO: logics for receive file */
            println!("Receiving file: from {:?} {} to my {}", peer, from, to);

            responder
                .send(ClientEvent::Consequence {
                    id,
                    consequence: Consequence::FileReceive { result: Ok(()) },
                })
                .await
        });
    }
}
