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
        p2p::p2p_test_run().await;
    }
}
