// 클라이언트를 바이너리로 실행
// 클라이언트 개발 테스트 용도
use dotenv::{dotenv, from_path};
use kudrive_client::{
    config_loader::get_config,
    event_loop, file_receive, file_send, init,
    p2p::{cli_helpfn, run_cli_command, P2PTransport},
    shutdown,
};
use std::{env, path::PathBuf};
use tokio::{
    io::{self, AsyncBufReadExt as _},
    sync::oneshot::Receiver,
};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    // run_p2p_cli().await;
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    init().await;

    if let Err(e) = event_loop().await {
        tracing::error!("Event loop error: {:?}", e);

        tracing::info!("Client shutdown");
        shutdown().await;
    }
}

// P2P Transport CLI Loop Test
async fn run_p2p_cli() {
    let _ = from_path(".client.env");
    dotenv().ok();

    let client_hostname = get_config().id.nickname.clone();
    let relay_address = get_config().server.p2p_relay_addr.clone();
    let (tx, rx) = tokio::sync::mpsc::channel(1024);
    let mut p2p_client = P2PTransport::new(
        &relay_address,
        &client_hostname,
        tx,
        PathBuf::from(get_config().file.workspace.clone()),
    )
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
                tracing::info!("Ctrl+C input Returning...");
                return;
            }
        }
    }
}
