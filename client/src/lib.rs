// lib.rs
pub mod client;
pub mod file_server;
pub mod net;
pub mod config_loader;

use std::{
    error::Error,
    sync::{Arc, LazyLock},
};

use client::Client;
use kudrive_common::event::{
    client::{ClientEvent, Command, Consequence},
    server::ClientMessage,
};
use tokio::sync::{oneshot, Mutex};

static GLOBAL_STATE: LazyLock<Arc<Mutex<Client>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Client::new())));

pub async fn init() {
    let mut client = GLOBAL_STATE.lock().await;
    client.start().await;
    drop(client);
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

    let client = GLOBAL_STATE.lock().await;
    client.sender().send(event).await.unwrap();
    drop(client);

    let handle = tokio::spawn(async move { receiver.await });
    match handle.await {
        Ok(Ok(consequence)) => Ok(consequence),
        Ok(Err(e)) => Err(format!("Failed to execute command {:?}", e)),
        Err(e) => Err(format!("Failed to execute command {:?}", e)),
    }
}

pub async fn file_send(target: String, from: String, to: String) -> Result<(), String> {
    let command = Command::FindPeer { target };

    let result = match execute_command(command).await {
        Ok(Consequence::FindPeer { result }) => result,
        Ok(_) => Err("Unexpected consequence".to_string()),
        Err(e) => Err(e),
    };

    match result {
        Ok(peer) => {
            let command = Command::FileSend { peer, from, to };

            match execute_command(command).await {
                Ok(Consequence::FileSend { result }) => result,
                Ok(_) => Err("Unexpected consequence".to_string()),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn file_receive(target: String, from: String, to: String) -> Result<(), String> {
    let command = Command::FindPeer { target };

    let result = match execute_command(command).await {
        Ok(Consequence::FindPeer { result }) => result,
        Ok(_) => Err("Unexpected consequence".to_string()),
        Err(e) => Err(e),
    };

    match result {
        Ok(peer) => {
            let command = Command::FileReceive { peer, from, to };

            match execute_command(command).await {
                Ok(Consequence::FileSend { result }) => result,
                Ok(_) => Err("Unexpected consequence".to_string()),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn clients() -> Result<(), String> {
    let command = Command::Clients {};

    match execute_command(command).await {
        Ok(Consequence::Clients { result }) => result,
        Ok(_) => Err("Unexpected consequence".to_string()),
        Err(e) => Err(e),
    }
}

pub async fn shutdown() {
    let mut client = GLOBAL_STATE.lock().await;
    client.shutdown().await;
    drop(client);
}

pub async fn send_event(event: ClientMessage) {
    let mut client = GLOBAL_STATE.lock().await;
    client.server.transmit(event).await.unwrap();
    drop(client);
}
