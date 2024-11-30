use std::collections::HashMap;
use tokio::net::TcpListener;
use uuid::Uuid;

mod client;
use client::ClientHandler;

struct MyServer {
    clients: HashMap<Option<Uuid>, Vec<ClientHandler>>,
}

impl MyServer {
    async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("127.0.0.1:7878").await?;
        println!("Server listening on port 7878");

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    println!("New client connected: {:?}", stream.peer_addr().unwrap());

                    let handler = ClientHandler::new(stream).await;
                    handler.start().await;

                    self.register(handler).await;
                }
                Err(e) => {
                    println!("Error occurred: {}", e);
                }
            }
        }
    }

    async fn register(&mut self, handler: ClientHandler) {
        let uuid = None;
        self.clients.entry(uuid).or_insert(vec![]).push(handler);
    }

    async fn group(&mut self, group: Uuid, id: Uuid) {
        todo!();
    }

    async fn remove(&mut self, group: Uuid, id: Uuid) {
        todo!();
    }
}

#[tokio::main]
async fn main() {
    let mut server = MyServer {
        clients: HashMap::new(),
    };

    let _ = server.init().await;

    println!("Done!");
}
