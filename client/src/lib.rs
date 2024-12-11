pub mod client;
pub mod config_loader;
pub mod event;
pub mod file_server;
pub mod net;

use std::{
    error::Error,
    sync::{Arc, LazyLock},
};

use client::handler::ClientHandler;
use event::{ClientEvent, Command, Consequence};
use tracing_subscriber::filter::LevelFilter;
use kudrive_common::{Client, Peer};
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;

static GLOBAL_STATE: LazyLock<Arc<Mutex<ClientHandler>>> =
    LazyLock::new(|| Arc::new(Mutex::new(ClientHandler::new())));
pub use net::p2p;

pub async fn init() {
    let _ = tracing_subscriber::fmt()
        // .with_max_level(LevelFilter::DEBUG)
        .with_env_filter(tracing_subscriber::EnvFilter::new("info"))
        .with_thread_names(true)
        .with_target(true)
        .try_init();

    let mut handler = GLOBAL_STATE.lock().await;
    handler.start().await;
    drop(handler);

    tokio::spawn(async move {

        match event_loop().await {
            Ok(_) => {
                tracing::info!("Event loop finished");
            }
            Err(e) => {
                tracing::error!("Event loop failed: {:?}", e);
            }
        };
    });
}

pub async fn event_loop() -> Result<(), Box<dyn Error>> {

    loop {
        let mut client = GLOBAL_STATE.lock().await;
        match client.event_listen().await {
            Ok(_) => {
                // tracing::info!("Event loop worked once");
            }
            Err(e) => {
                return Err(Box::new(e));
            }
        }
        drop(client);
    }
}

pub async fn execute_command(command: Command) -> Result<Consequence, String> {
    let (responder, receiver) = oneshot::channel::<Consequence>();

    let event = ClientEvent::Command { command, responder };

    let handler = GLOBAL_STATE.lock().await;
    handler.sender().send(event).await.unwrap();
    drop(handler);

    let handle = tokio::spawn(async move { receiver.await });
    match handle.await {
        Ok(Ok(consequence)) => Ok(consequence),
        Ok(Err(e)) => Err(format!("Failed to execute command {:?}", e)),
        Err(e) => Err(format!("Failed to execute command {:?}", e)),
    }
}

pub async fn file_send(id: Uuid, source: String, target: String) -> Result<(), String> {
    let peer = Peer { id, source, target };
    let command = Command::FileSend { peer: peer.clone() };

    match execute_command(command).await {
        Ok(Consequence::FileSend { result }) => result,
        Ok(_) => Err("Unexpected consequence".to_string()),
        Err(e) => Err(e),
    }
}

pub async fn file_receive(id: Uuid, source: String, target: String) -> Result<(), String> {
    let peer = Peer { id, source, target };
    let command = Command::FileReceive { peer };

    match execute_command(command).await {
        Ok(Consequence::FileReceive { result }) => result,
        Ok(_) => Err("Unexpected consequence".to_string()),
        Err(e) => Err(e),
    }
}

pub async fn clients() -> Result<Vec<Client>, String> {
    let command = Command::Clients {};

    match execute_command(command).await {
        Ok(Consequence::Clients { result }) => result,
        Ok(_) => Err("Unexpected consequence".to_string()),
        Err(e) => Err(e),
    }
}

pub async fn shutdown() {
    let mut handler = GLOBAL_STATE.lock().await;
    handler.shutdown().await;
    drop(handler);
}
