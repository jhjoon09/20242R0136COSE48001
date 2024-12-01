// 클라이언트를 바이너리로 실행
// 클라이언트 개발 테스트 용도
use kudrive_client::{event_loop, init, shutdown};

#[tokio::main]
async fn main() {
    init().await;

    if let Err(e) = event_loop().await {
        eprintln!("Event loop error: {:?}", e);

        println!("Client shutdown");
        shutdown().await;
    }

    // P2P Transport CLI Loop Test
    let _ = from_path(".client.env");
    dotenv().ok();

    let _ = Client::new();

    let client_hostname = env::var("CLIENT_NAME").expect("CLIENT_NAME must be set");
    let relay_address = env::var("RELAY_ADDR").expect("RELAY_ADDR must be set");
    let mut p2p_client = P2pTransport::new(&relay_address, &client_hostname)
        .await
        .expect("Failed to create P2P client");
    let _ = p2p_client.connect_relay(10).await;
    let _ = p2p_client.listen_on_peer(10).await;
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    let mut send_rx: Option<Receiver<Result<(), String>>> = None;
    cli_helpfn();
    loop {
        tokio::select! {
            Ok(Some(line)) = stdin.next_line() => {
                if run_cli_command(&mut p2p_client, &line, &mut send_rx).await {
                    break;
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Ctrl+C input Returning...");
                return;
            }
        }
    }
}
