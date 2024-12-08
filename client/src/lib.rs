// lib.rs
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
use kudrive_common::{message::client::ClientMessage, Client};
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;

static GLOBAL_STATE: LazyLock<Arc<Mutex<ClientHandler>>> =
    LazyLock::new(|| Arc::new(Mutex::new(ClientHandler::new())));

pub async fn init() {
    let mut handler = GLOBAL_STATE.lock().await;
    handler.start().await;
    drop(handler);

    // tokio::spawn(async move {
    //     event_loop().await.unwrap();
    // });
}

pub async fn event_loop() -> Result<(), Box<dyn Error>> {
    // TODO: implement server health check
    // TODO: implement server reconnect
    // TODO: implement health check message send
    loop {
        let mut client = GLOBAL_STATE.lock().await;
        if let Err(e) = client.event_listen().await {
            return Err(Box::new(e));
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
    Ok(())
}

pub async fn file_receive(id: Uuid, source: String, target: String) -> Result<(), String> {
    Ok(())
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
