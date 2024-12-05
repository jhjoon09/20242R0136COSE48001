use std::sync::Arc;

use kudrive_common::{
    event::server::{ClientMessage, MetaEvent, ServerEvent},
    tcp::transmitter,
    Client, Listener, Transmitter,
};
use tokio::{
    net::TcpStream,
    sync::{
        mpsc::{self, error::TryRecvError, Receiver, Sender},
        Mutex,
    },
};

pub struct ClientHandler {
    info: Option<Client>,
    meta: Sender<MetaEvent>,
    receiver: Receiver<ServerEvent>,
    transmitter: Transmitter,
    listener: Listener<ClientMessage, ServerEvent>,
}

impl ClientHandler {
    pub fn new(stream: TcpStream, meta: mpsc::Sender<MetaEvent>) -> Self {
        let stream = Arc::new(Mutex::new(stream));
        let (sender, receiver) = mpsc::channel::<ServerEvent>(1024);

        let transmitter = transmitter::Transmitter::new(stream.clone());
        let listener = Listener::spawn(stream.clone(), sender.clone());

        Self {
            info: None,
            meta,
            receiver,
            transmitter,
            listener,
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
            _ => {}
        };

        Ok(())
    }
}
