use std::sync::Arc;

use bytes::{BufMut, BytesMut};
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::message::Message;

pub struct Transmitter {
    stream: Arc<Mutex<TcpStream>>,
}

impl Transmitter {
    pub fn new(stream: Arc<Mutex<TcpStream>>) -> Self {
        Self { stream }
    }

    pub async fn send(&self, message: impl Message) -> io::Result<()> {
        let bytes = message.to_bytes();
        let length = bytes.len() as u32;

        let mut buffer = BytesMut::with_capacity(4 + length as usize);
        buffer.put_u32_le(length);
        buffer.extend_from_slice(&bytes);

        let mut lock = self.stream.lock().await;
        lock.write_all(&buffer).await?;
        drop(lock);

        Ok(())
    }
}
