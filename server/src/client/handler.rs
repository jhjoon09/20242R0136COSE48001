use std::sync::Arc;

use kudrive_common::{
    message::client::ClientMessage, tcp::transmitter, Client, FileMap, Listener, Transmitter,
};
use tokio::{
    net::TcpStream,
    sync::{
        mpsc::{self, error::TryRecvError, Receiver, Sender},
        Mutex, RwLock,
    },
};

use crate::event::{MetaEvent, ServerEvent};

use super::group::ClientGroup;

pub struct ClientHandler {
    client: Option<Client>,
    group: Option<Arc<RwLock<ClientGroup>>>,
    meta: Sender<MetaEvent>,
    sender: Sender<ServerEvent>,
    receiver: Receiver<ServerEvent>,
    transmitter: Transmitter,
}

impl ClientHandler {
    pub fn new(stream: TcpStream, meta: mpsc::Sender<MetaEvent>) -> Self {
        let stream = Arc::new(Mutex::new(stream));
        let (sender, receiver) = mpsc::channel::<ServerEvent>(1024);

        let transmitter = transmitter::Transmitter::new(stream.clone());
        let _listener = Listener::spawn(stream.clone(), sender.clone());

        Self {
            client: None,
            group: None,
            meta,
            sender,
            receiver,
            transmitter,
        }
    }

    fn sender(&self) -> Sender<ServerEvent> {
        self.sender.clone()
    }

    async fn register(&mut self, client: Client) {
        let Client { group, id, .. } = client.clone();
        let event = MetaEvent::Register { group, id };

        self.info = Some(client);
        self.meta.send(event).await.unwrap();
    }

    async fn update(&mut self, file_map: FileMap) {
        if let Some(ref client) = self.info {
            let event = MetaEvent::Propagation {
                group: client.group,
            };

            self.info = Some(Client {
                files: file_map,
                ..client.clone()
            });

            self.meta.send(event).await.unwrap()
        }
    }

    fn try_receive(&mut self) -> Result<ServerEvent, TryRecvError> {
        self.receiver.try_recv()
    }

    pub async fn event_listen(&mut self) -> Result<(), TryRecvError> {
        let event = match self.try_receive() {
            Ok(event) => event,
            Err(TryRecvError::Empty) => return Ok(()),
            Err(e) => return Err(e),
        };

        match event {
            ServerEvent::Message { message } => match message {
                ClientMessage::Register { client } => {
                    self.register(client).await;
                }
                ClientMessage::FileMapUpdate { file_map } => {
                    self.update(file_map).await;
                }
                _ => {}
            },
        };

        Ok(())
    }
}
