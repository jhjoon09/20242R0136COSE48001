use std::sync::Arc;

use bytes::{Buf, BytesMut};
use tokio::io::{self};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};

use crate::message::Message;

pub struct Listener<T: Message> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: Message> Listener<T> {
    pub fn spawn(stream: Arc<Mutex<TcpStream>>, sender: mpsc::Sender<T>) -> Self {
        tokio::spawn(async move {
            if let Err(e) = handle_stream(stream, sender).await {
                eprintln!("Failed to handle stream: {}", e);
            }
        });
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

async fn handle_stream<T: Message>(
    stream: Arc<Mutex<TcpStream>>,
    sender: mpsc::Sender<T>,
) -> io::Result<()> {
    let mut buffer = BytesMut::new();
    let mut next: Option<usize> = None;

    loop {
        let mut tmp = vec![0; 1024];
        let lock = stream.lock().await;
        let read_res = lock.try_read(&mut tmp);
        drop(lock);

        match read_res {
            // connection closed
            Ok(0) => break,
            // data received
            Ok(n) => buffer.extend_from_slice(&tmp[..n]),
            // not readable yet
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
            // unexpected error
            Err(e) => return Err(e),
        };

        loop {
            match next {
                // enough to read next message
                Some(len) if buffer.len() >= len => {
                    let data = buffer.split_to(len).freeze();
                    let message = Message::from_bytes(&data);
                    if sender.send(message).await.is_err() {
                        eprintln!("Failed to send message to the receiver");
                    }
                    next = None;
                }
                // more message exists in the buffer
                None if buffer.len() >= 4 => {
                    let length = buffer.get_u32_le() as usize;
                    next = Some(length);
                }
                // not enough data
                _ => break,
            }
        }
    }
    Ok(())
}
