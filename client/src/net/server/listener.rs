use std::sync::Arc;

use tokio::io::{self};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};

pub struct Listener {}

impl Listener {
    pub fn spawn(stream: Arc<Mutex<TcpStream>>, sender: mpsc::Sender<String>) -> Self {
        tokio::spawn(async move {
            if let Err(e) = handle_stream(stream, sender).await {
                eprintln!("Failed to handle stream: {}", e);
            }
        });
        Self {}
    }
}

async fn handle_stream(
    stream: Arc<Mutex<TcpStream>>,
    sender: mpsc::Sender<String>,
) -> io::Result<()> {
    let mut buffer = vec![0; 1024];
    loop {
        let lock = stream.lock().await;
        let read_res = lock.try_read(&mut buffer);
        drop(lock);

        match read_res {
            // connection closed
            Ok(0) => break,
            // data received
            Ok(n) => {
                let message = String::from_utf8_lossy(&buffer[..n]).to_string();
                let send_res = sender.send(message).await;
                match send_res {
                    // receiver dropped
                    Err(_) => break,
                    // message sent
                    Ok(_) => continue,
                }
            }
            // not readable yet
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
            // unexpected error
            Err(e) => return Err(e),
        };
    }
    Ok(())
}
