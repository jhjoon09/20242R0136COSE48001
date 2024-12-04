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
        tracing::info!("Server started.");
        // let _ = p2p::P2PTransport::run(4001, false).await;
        let (exit_tx, exit_rx) = tokio::sync::oneshot::channel();
        tokio::task::spawn(async {
            let _ = p2p::P2PTransport::run_with_restart(4001, 10, exit_rx).await;
        });
        println!("Server started. Press Enter to exit.");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let _ = exit_tx.send(());

        tracing::info!("Server exited.");
    }
}
