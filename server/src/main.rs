use kudrive_server::Server;
pub mod p2p;
use clap::Parser;

#[derive(Debug, Parser)]
struct Opts {
    #[clap(long)]
    test_p2p: bool,
}

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();
    if opts.test_p2p {
        let _ = p2p::P2PTransport::run(4001, false).await;
    } else {
        tokio::task::spawn(async {
            let (tx, exit_rx) = tokio::sync::oneshot::channel();
            let _ = p2p::P2PTransport::run_with_restart(4001, 60 * 60, exit_rx).await;
        });

        let mut server = Server::new().await;

        server.start().await.unwrap();

        println!("Done!");

        tracing::info!("Server started.");
    }
}
