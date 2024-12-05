use kudrive_server::Server;

#[tokio::main]
async fn main() {
    let mut server = Server::new().await;

    server.start().await.unwrap();

    println!("Done!");
}
