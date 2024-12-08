use futures::StreamExt;
use kudrive_common::p2p::generate_ed25519;
use libp2p::{
    core::{multiaddr::Protocol, Multiaddr},
    dcutr, identify, identity, noise, ping, relay,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    tcp, yamux,
};
use std::{error::Error, net::Ipv4Addr, sync::Arc};
use tokio::{
    io::{self, AsyncBufReadExt as _},
    select,
    sync::{mpsc, oneshot, Mutex},
};
use tracing_subscriber::EnvFilter;

const RELAY_ID: &str = "0";

#[derive(NetworkBehaviour)]
struct Behaviour {
    relay: relay::Behaviour,
    ping: ping::Behaviour,
    identify: identify::Behaviour,
    dcutr: dcutr::Behaviour,
}

pub enum P2PCommand {
    Exit,
    StrMsg(String),
}

pub struct P2PTransport {
    port: u16,
    command_tx: mpsc::Sender<P2PCommand>,
    restart_exit_tx: Option<oneshot::Sender<()>>,
}

impl P2PTransport {
    fn new(port: u16) -> Self {
        let (command_tx, command_rx) = mpsc::channel(32);
        let port_clone = port;
        tokio::spawn(async move {
            let _ = Self::start_swarm(command_rx, port_clone).await;
        });

        Self {
            port,
            command_tx,
            restart_exit_tx: None,
        }
    }

    pub async fn run(port: u16, use_cli: bool) {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .try_init();
        if use_cli {
            let transport: P2PTransport = Self::new(port);
            transport.run_with_cli().await;
        } else {
            let _ = Self::new(port);
        }
    }

    pub async fn shutdown(&mut self) {
        let _ = self.command_tx.send(P2PCommand::Exit).await;
        if let Some(tx) = self.restart_exit_tx.take() {
            let _ = tx.send(());
        }
    }

    async fn run_with_cli(&self) {
        let mut stdin = io::BufReader::new(io::stdin()).lines();
        loop {
            select! {
                Ok(Some(line)) = stdin.next_line() => {
                    if let Ok(is_end) = self.handle_cli_command(&line).await {
                        if is_end {
                            return;
                        }
                    }
                },
                _ = tokio::signal::ctrl_c() => {
                    let _ = self.command_tx.send(P2PCommand::Exit).await;
                    tracing::info!("Ctrl+C received. Exiting...");
                    return;
                }
            }
        }
    }

    pub async fn run_with_restart(
        port: u16,
        restart_interval: u64,
        mut exit_rx: oneshot::Receiver<()>,
    ) {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .try_init();

        loop {
            tracing::info!("Starting swarm on port {}...", port);
            let (command_tx, command_rx) = mpsc::channel(32);
            let swarm_handle = match Self::start_swarm(command_rx, port).await {
                Ok(handle) => handle,
                Err(e) => {
                    tracing::error!("Failed to start swarm: {}", e);
                    break;
                }
            };

            tracing::info!(
                "Swarm started on port {}. Running for {} seconds.",
                port,
                restart_interval
            );

            tokio::select! {
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(restart_interval)) => {
                    tracing::info!("Restarting swarm...");
                },
                Ok(_) = &mut exit_rx => {
                    tracing::info!("Received exit signal, shutting down...");
                    break;
                }
            }

            if let Err(e) = command_tx.send(P2PCommand::Exit).await {
                tracing::warn!("Failed to send exit command to swarm: {}", e);
            }
            swarm_handle.lock().await.shutdown().await;
            tracing::info!("Swarm shutdown complete.");
        }
    }

    async fn start_swarm(
        event_rx: mpsc::Receiver<P2PCommand>,
        port: u16,
    ) -> Result<Arc<Mutex<SwarmHandle>>, Box<dyn Error>> {
        let local_key: identity::Keypair = generate_ed25519(RELAY_ID);

        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|key| Behaviour {
                relay: relay::Behaviour::new(key.public().to_peer_id(), Default::default()),
                ping: ping::Behaviour::new(ping::Config::new()),
                identify: identify::Behaviour::new(identify::Config::new(
                    "/KUDRIVE/0.0.1".to_string(),
                    key.public(),
                )),
                dcutr: dcutr::Behaviour::new(key.public().to_peer_id()),
            })?
            .build();

        let listen_addr_tcp = Multiaddr::empty()
            .with(Protocol::from(Ipv4Addr::UNSPECIFIED))
            .with(Protocol::Tcp(port));
        swarm.listen_on(listen_addr_tcp.clone())?;

        let swarm_handle = Arc::new(Mutex::new(SwarmHandle::new(swarm, event_rx)));

        let swarm_handle_clone = Arc::clone(&swarm_handle);
        tokio::spawn(async move {
            swarm_handle_clone.lock().await.run().await;
        });

        Ok(swarm_handle)
    }

    async fn handle_cli_command(&self, cmd: &str) -> Result<bool, Box<dyn Error>> {
        match cmd {
            "exit" => {
                self.command_tx.send(P2PCommand::Exit).await?;
                return Ok(true);
            }
            "" => {}
            _ => {
                self.command_tx
                    .send(P2PCommand::StrMsg(cmd.to_string()))
                    .await?;
            }
        }
        Ok(false)
    }
}

pub struct SwarmHandle {
    swarm: Option<Swarm<Behaviour>>,
    event_rx: Option<mpsc::Receiver<P2PCommand>>,
}

impl SwarmHandle {
    fn new(swarm: Swarm<Behaviour>, event_rx: mpsc::Receiver<P2PCommand>) -> Self {
        Self {
            swarm: Some(swarm),
            event_rx: Some(event_rx),
        }
    }

    pub async fn run(&mut self) {
        if self.swarm.is_none() || self.event_rx.is_none() {
            return;
        }
        let mut swarm = self.swarm.take().expect("Swarm should exist");
        let mut event_rx = self.event_rx.take().expect("Receiver should exist");

        loop {
            select! {
                Some(swarm_event) = swarm.next() => {
                    match swarm_event {
                        SwarmEvent::Behaviour(event) => {
                            tracing::info!("{:?}", event);
                        }
                        SwarmEvent::NewListenAddr { address, .. } => {
                            tracing::info!("Listening on {}", address);
                        }
                        _ => {}
                    }
                },
                Some(msg) = event_rx.recv() => {
                    if let P2PCommand::Exit = msg {
                        tracing::info!("Exiting swarm...");
                        break;
                    }
                }
            }
        }
    }

    pub async fn shutdown(&mut self) {
        self.event_rx.take();
        self.swarm.take();
    }
}
