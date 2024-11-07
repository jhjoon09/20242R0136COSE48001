use kudrive_common::{ClientInfo, DirTree, Server};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

struct MyServer {
    clients: Arc<RwLock<HashMap<ClientInfo, DirTree>>>,
}

impl MyServer {
    async fn init(&self) {
        println!("Starting server...");
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        println!("Server shutdown...");
    }
}

#[tokio::main]
async fn main() {
    let server = MyServer {
        clients: Arc::new(RwLock::new(HashMap::new())),
    };

    server.init().await;

    println!("Done!");
}
