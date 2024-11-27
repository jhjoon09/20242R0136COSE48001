// lib.rs
pub mod client;
pub mod file_server;
pub mod net;

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
    loop {
        let mut client = GLOBAL_STATE.lock().await;
        if let Err(e) = client.event_listen().await {
            return Err(Box::new(e));
        }
        drop(client);
    }
}

pub async fn file_send(target: String, from: String, to: String) -> Result<(), String> {
    let (responder, receiver) = oneshot::channel::<Consequence>();

    let command = Command::FileSend { target, from, to };
    let event = ClientEvent::Command { command, responder };

    let client = GLOBAL_STATE.lock().await;
    client.sender().send(event).await.unwrap();
    drop(client);

    let handle = tokio::spawn(async move { receiver.await });
    println!("File send request sent.");

    // 태스크 완료를 기다림
    match handle.await {
        Ok(Ok(consequence)) => match consequence {
            Consequence::FileSend { result } => result,
            _ => Err("Unexpected consequence".to_string()),
        },
        Ok(Err(e)) => Err(format!("Failed to send file: {:?}", e)),
        Err(e) => Err(format!("Failed to send file: {:?}", e)),
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
