use tokio::{sync::oneshot, time};

use kudrive_common::Peer;
use tokio::sync::mpsc::Sender;

use crate::event::{ClientEvent, Consequence};

use futures::{executor::block_on, future::FutureExt, stream::StreamExt};
use kudrive_common::p2p::generate_ed25519;
use libp2p::{
    core::multiaddr::{Multiaddr, Protocol},
    dcutr, identify, noise, ping, relay,
    request_response::{self, ProtocolSupport},
    swarm::{DialError, NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, StreamProtocol, Swarm,
};
use std::{collections::HashMap, num::NonZero, path::PathBuf, str::FromStr};
use std::{error::Error, time::Duration};
use tokio::{
    fs::File,
    io::AsyncReadExt,
    select,
    sync::{
        mpsc::{self, Receiver, Sender},
        oneshot,
    },
};
use tracing_subscriber::EnvFilter;

use super::ku_protocol::{self, KuFileTransferCodec};

// Swarm config
const SWARM_IDLE_TIMEOUT: u64 = 60;
const SWARM_CONN_BUF_SIZE: usize = 1000;
const SWARM_MAX_NEGOTIATING_INBOUND_STREAMS: usize = 100;
const SWARM_NOTIFY_BUF_SIZE: usize = 1000;

// P2PTransport config
const MAX_DIAL_RETRY: usize = 3;
const INIT_LISTEN_DELAY: u64 = 2;
const CMD_BUFF_SIZE: usize = 10000;
const REQUEST_TIMEOUT_SEC: u64 = 50;

#[derive(Clone, Debug, PartialEq)]
pub enum P2pStatus {
    NotConnected,
    RelayConnected,
    PeerConnected(Vec<String>),
}

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    relay_client: relay::client::Behaviour,
    ping: ping::Behaviour,
    identify: identify::Behaviour,
    dcutr: dcutr::Behaviour,
    ku_file_transfer: request_response::Behaviour<KuFileTransferCodec>,
    // ku_messaging: request_response::Behaviour<MessagingCodec>,
}

pub enum P2pCommand {
    Exit,
    GetPendingRequests {
        response_tx: oneshot::Sender<Vec<String>>,
    },
    GetId {
        response_tx: oneshot::Sender<String>,
    },
    GetStatus {
        response_tx: oneshot::Sender<P2pStatus>,
    },
    GetListenAddr {
        response_tx: oneshot::Sender<Vec<String>>,
    },
    ConnectToRelay {
        response_tx: oneshot::Sender<Result<(), String>>,
    },
    ListenToPeer {
        response_tx: oneshot::Sender<Result<(), String>>,
    },
    ConnectToPeer {
        remote_peer_id: PeerId,
        response_tx: oneshot::Sender<Result<(), String>>,
    },
    RecvFile {
        remote_peer_id: String,
        src_path: String,
        tgt_path: String,
        response_tx: oneshot::Sender<Result<(), String>>,
    },
    SendFileOpen {
        src_path: String,
        response_tx: oneshot::Sender<Result<(), String>>,
    },
}

pub struct P2PTransport {
    pub p2p_id: PeerId,
    pub relay_address: Multiaddr,
    pub command_tx: Sender<P2pCommand>,
    responder: Sender<ClientEvent>,
}

impl P2PTransport {
    pub fn new_mock(responder: Sender<ClientEvent>) -> Self {
        let (tx, _) = mpsc::channel::<P2pCommand>(CMD_BUFF_SIZE);
        Self {
            p2p_id: PeerId::random(),
            relay_address: Multiaddr::empty(),
            command_tx: tx,
            responder
        }
    }
    
    fn responder(&self) -> Sender<ClientEvent> {
        self.responder.clone()
    }

    pub async fn new(relay_address: &str, secret_key_seed: &str, responder: Sender<ClientEvent>) -> Result<Self, Box<dyn Error>> {
        let relay_address = Multiaddr::from_str(relay_address).expect("Invalid relay address");
        let mut swarm = Self::init_swarm(secret_key_seed, &mut relay_address.clone()).await?;
        let p2p_id = swarm.local_peer_id().clone();

        let (tx, rx) = mpsc::channel::<P2pCommand>(CMD_BUFF_SIZE);
      
        let p2p_client = Self {
            p2p_id,
            relay_address,
            command_tx: tx,
            responder,
        };
        let relay_address = p2p_client.relay_address.clone();

        if !Self::is_relay_connected(&swarm, &relay_address.to_string()) {
            if let Ok(()) = Self::dial_relay(&mut swarm, &relay_address).await {
            } else {
                tracing::error!("Failed to dial to relay ");
            }
        }

        if !Self::is_listen_peer_via_relay(&mut swarm) {
            for _ in 0..MAX_DIAL_RETRY {
                if Self::listen_peer_via_relay(&mut swarm, &relay_address)
                    .await
                    .is_ok()
                {
                    break;
                }
            }
            if !Self::is_listen_peer_via_relay(&mut swarm) {
                tracing::error!("Failed to listen via relay: {:?}", &relay_address);
            }
        }

        tokio::task::spawn(async move {
            Self::swarm_event_loop(
                swarm,
                rx,
                // base_dir_path,
                relay_address,
            )
            .await;
        });
        Ok(p2p_client)
    }

    pub async fn send_open(&self, own: bool, pending: u64, peer: Peer) {
        let responder = self.responder();

        // tokio::spawn(async move {
        //     /* START TEMP */
        //     println!("Opening for sending file: {}", peer.source);
        //     time::sleep(time::Duration::from_secs(5)).await;
        //     println!("Opened for sending file: {}", peer.source);
        //     let (tx, rx) = oneshot::channel();
        //     /* END TEMP */

        //     let ids = match own {
        //         true => (Some(pending), None),
        //         false => (None, Some(pending)),
        //     };

        //     let convey = (peer, rx);
        //     let event = ClientEvent::Opened { ids, convey };
        //     responder.send(event).await.unwrap();
        // });
        ()
    }

    pub async fn send_wait(
        &self,
        pending: Option<u64>,
        peer: Peer,
        rx: oneshot::Receiver<Result<(), String>>,
    ) {
        let responder = self.responder();

        // tokio::spawn(async move {
        //     /* START TEMP */
        //     println!("Waiting for sending file: {}", peer.source);
        //     time::sleep(time::Duration::from_secs(5)).await;
        //     println!("Waited for sending file: {}", peer.source);
        //     /* END TEMP */

        //     if let Some(id) = pending {
        //         let consequence = Consequence::FileSend { result: Ok(()) };
        //         let event = ClientEvent::Consequence { id, consequence };
        //         responder.send(event).await.unwrap();
        //     }
        // });
        ()
    }

    pub async fn receive(&self, pending: Option<u64>, peer: Peer) {
        let responder = self.responder();

        // tokio::spawn(async move {
        //     println!(
        //         "Receiving file from {}: {} -> {}",
        //         peer.id, peer.source, peer.target
        //     );
        //     time::sleep(time::Duration::from_secs(5)).await;
        //     println!(
        //         "Received file from {}: {} -> {}",
        //         peer.id, peer.source, peer.target
        //     );
        //     /* END TEMP */

        //     if let Some(id) = pending {
        //         let consequence = Consequence::FileReceive { result: Ok(()) };
        //         let event = ClientEvent::Consequence { id, consequence };
        //         responder.send(event).await.unwrap();
        //     }
        // });
        ()
    }
    

    pub async fn exit(&self) -> Result<(), Box<dyn Error>> {
        let command = P2pCommand::Exit;
        self.command_tx.send(command).await?;
        Ok(())
    }

    pub async fn warm_up_with_delay(&self, delay: u64) -> Result<(), Box<dyn Error>> {
        let mut time_left = delay;

        let mut cur_time = tokio::time::Instant::now();
        let _ = self.connect_relay(2).await;
        time_left = time_left.saturating_sub(cur_time.elapsed().as_secs());
        tracing::debug!("Relay connect took {:?} sec", cur_time.elapsed());

        tokio::time::sleep(Duration::from_secs(delay / 2)).await;

        cur_time = tokio::time::Instant::now();
        let _ = self.listen_on_peer(2).await;
        tracing::debug!("Listening took {:?} sec", cur_time.elapsed());

        tokio::time::sleep(Duration::from_secs(delay - (delay / 2))).await;
        Ok(())
    }

    pub async fn recv_file(
        &self,
        remote_peer_id: String,
        target_path: String,
        save_path: String,
        timeout: u64,
    ) -> Result<(), Box<dyn Error>> {
        let (tx, rx) = oneshot::channel();
        let command = P2pCommand::RecvFile {
            remote_peer_id: remote_peer_id.clone(),
            src_path: target_path.clone(),
            tgt_path: save_path,
            response_tx: tx,
        };

        self.connect_relay(timeout).await?;
        self.connect_peer(remote_peer_id.clone(), timeout).await?;

        tracing::info!("Sending file request to cmd q: {:?}", target_path);
        self.command_tx.send(command).await?;
        tracing::info!("Sent file request to cmd q: {:?}", target_path);
        tracing::info!("Waiting for file request response: {:?}", target_path);
        tokio::select! {
            res = rx => {
                match res {
                    Ok(Ok(())) => Ok(()),
                    Ok(Err(err)) => Err(err.into()),
                    Err(recv_err) => Err(recv_err.into()),
                }
            },
            _ = tokio::time::sleep(Duration::from_secs(REQUEST_TIMEOUT_SEC)) => {
                Err("Timeout while waiting for file request response".into())
            }
        }
    }

    pub async fn send_file_open(
        &self,
        src_path: String,
    ) -> Result<oneshot::Receiver<Result<(), String>>, Box<dyn Error>> {
        self.listen_on_peer(REQUEST_TIMEOUT_SEC).await?;
        let (tx, rx) = oneshot::channel();
        let command = P2pCommand::SendFileOpen {
            src_path,
            response_tx: tx,
        };
        self.command_tx.send(command).await?;
        Ok(rx)
    }

    pub async fn send_file_wait(
        &self,
        rx: &mut oneshot::Receiver<Result<(), String>>,
        timeout: u64,
    ) -> Result<(), Box<dyn Error>> {
        tokio::select! {
            res = rx => {
                match res {
                    Ok(Ok(())) => Ok(()),
                    Ok(Err(err)) => Err(err.into()),
                    Err(recv_err) => Err(recv_err.into()),
                }
            },
            _ = tokio::time::sleep(Duration::from_secs(timeout)) => {
                Err("Timeout while waiting for file send response".into())
            }
        }
    }

    pub async fn connect_peer(&self, peer_id: String, timeout: u64) -> Result<(), Box<dyn Error>> {
        let (tx, rx) = oneshot::channel();
        let command = P2pCommand::ConnectToPeer {
            remote_peer_id: PeerId::from_str(&peer_id)?,
            response_tx: tx,
        };
        self.command_tx.send(command).await?;
        tokio::select! {
            res = rx => {
                match res {
                    Ok(Ok(())) => Ok(()),
                    Ok(Err(err)) => Err(err.into()),
                    Err(recv_err) => Err(recv_err.into()),
                }
            },
            _ = tokio::time::sleep(Duration::from_secs(timeout)) => {
                Err("Timeout while waiting for connect peer response".into())
            }
        }
    }

    pub async fn connect_relay(&self, timeout: u64) -> Result<(), Box<dyn Error>> {
        let (tx, rx) = oneshot::channel();
        let command = P2pCommand::ConnectToRelay { response_tx: tx };
        self.command_tx.send(command).await?;
        tokio::select! {
            res = rx => {
                match res {
                    Ok(Ok(())) => Ok(()),
                    Ok(Err(err)) => Err(err.into()),
                    Err(recv_err) => Err(recv_err.into()),
                }
            },
            _ = tokio::time::sleep(Duration::from_secs(timeout)) => {
                Err("Timeout while waiting for connect relay response".into())
            }
        }
    }

    pub async fn listen_on_peer(&self, timeout: u64) -> Result<(), Box<dyn Error>> {
        let (tx, rx) = oneshot::channel();
        let command = P2pCommand::ListenToPeer { response_tx: tx };
        self.command_tx.send(command).await?;
        tokio::select! {
            res = rx => {
                match res {
                    Ok(_) => return Ok(()),
                    _ => {
                        tracing::error!("Failed to listen on peer");
                        return Err("Failed to listen on peer".into());
                    }
                }
            },
            _ = tokio::time::sleep(Duration::from_secs(timeout)) => {
                tracing::error!("Timeout while waiting for listen on peer response");
                return Err("Timeout while waiting for listen on peer response".into());
            }
        }
    }

    pub async fn get_listen_addr(&self, timeout: u64) -> Result<Vec<String>, Box<dyn Error>> {
        let (tx, rx) = oneshot::channel();
        let command = P2pCommand::GetListenAddr { response_tx: tx };
        self.command_tx.send(command).await?;
        tokio::select! {
            res = rx => {
                match res {
                    Ok(res) => return Ok(res),
                    _ => {
                        tracing::error!("Failed to listen on peer");
                        return Err("Failed to listen on peer".into());
                    }
                }
            },
            _ = tokio::time::sleep(Duration::from_secs(timeout)) => {
                tracing::error!("Timeout while waiting for listen on peer response");
                return Err("Timeout while waiting for listen on peer response".into());
            }
        }
    }

    pub async fn is_listening(&self, timeout: u64) -> Result<bool, Box<dyn Error>> {
        let listen_addrs = self.get_listen_addr(timeout).await?;
        Ok(listen_addrs.iter().any(|addr| addr.contains("p2p-circuit")))
    }

    async fn swarm_event_loop(
        mut swarm: Swarm<Behaviour>,
        mut command_rx: Receiver<P2pCommand>,
        // base_dir_path: std::path::PathBuf,
        mut relay_addr: Multiaddr,
    ) {
        let mut pending_requests: HashMap<String, oneshot::Sender<Result<(), String>>> =
            HashMap::new();
        let mut told_relay_observed_addr = false;
        let mut learned_observed_addr = false;
        let mut is_exit = false;
        // let mut base_dir_path = base_dir_path.clone();
        loop {
            select! {
                Some(command) = command_rx.recv() => {
                    Self::handle_command(&mut swarm, command, &mut pending_requests, &mut relay_addr, &mut is_exit).await;
                }
                Some(event) = swarm.next() => {
                    let _ = Self::handle_swarm_event(
                        &mut swarm,
                        &relay_addr,
                        &mut told_relay_observed_addr,
                        &mut learned_observed_addr,
                        event,
                        // base_dir_path.clone(),
                        &mut pending_requests
                    ).await;
                }
                else => {
                    tracing::info!("EventLoop closing. Exiting swarm_event_loop.");
                    break;
                }
            }
            if is_exit {
                break;
            }
        }
    }

    async fn handle_command(
        swarm: &mut Swarm<Behaviour>,
        command: P2pCommand,
        pending_requests: &mut HashMap<String, oneshot::Sender<Result<(), String>>>,
        relay_addr: &mut Multiaddr,
        is_exit: &mut bool,
    ) {
        match command {
            P2pCommand::Exit => {
                tracing::info!("Received Exit command");
                *is_exit = true;
            }
            P2pCommand::GetPendingRequests { response_tx } => {
                let pending_requests: Vec<String> = pending_requests.keys().cloned().collect();
                tracing::info!("Pending request----");
                for pending_request in &pending_requests {
                    tracing::info!("Pending request: {:?}", pending_request);
                }
                let _ = response_tx.send(pending_requests);
            }
            P2pCommand::GetId { response_tx } => {
                tracing::info!("Received GetId command");
                let _ = response_tx.send(swarm.local_peer_id().clone().to_string());
            }
            P2pCommand::GetStatus { response_tx } => {
                tracing::info!("Received GetStatus command");
                let status = if Self::is_relay_connected(&swarm, &relay_addr.to_string()) {
                    let peer_ids: Vec<String> = swarm
                        .connected_peers()
                        .filter(|peer_id| peer_id.to_string() != relay_addr.to_string())
                        .map(|peer_id| peer_id.to_string())
                        .collect();
                    if peer_ids.is_empty() {
                        P2pStatus::RelayConnected
                    } else {
                        P2pStatus::PeerConnected(peer_ids)
                    }
                } else {
                    P2pStatus::NotConnected
                };
                let _ = response_tx.send(status);
            }
            P2pCommand::GetListenAddr { response_tx } => {
                let listen_addrs: Vec<String> =
                    swarm.listeners().map(|addr| addr.to_string()).collect();
                let _ = response_tx.send(listen_addrs);
            }
            P2pCommand::ConnectToRelay { response_tx } => {
                if Self::is_relay_connected(&swarm, &relay_addr.to_string()) {
                    let _ = response_tx.send(Ok(()));
                    return;
                } else {
                    if let Ok(()) = Self::dial_relay(swarm, &relay_addr).await {
                        pending_requests.insert(relay_addr.to_string(), response_tx);
                    } else {
                        tracing::error!("Failed to dial to relay ");
                        let _ = response_tx.send(Err("Failed to dial to relay".into()));
                    }
                }
            }
            P2pCommand::ListenToPeer { response_tx } => {
                if !Self::is_listen_peer_via_relay(&swarm) {
                    for _ in 0..MAX_DIAL_RETRY {
                        if Self::listen_peer_via_relay(swarm, relay_addr).await.is_ok() {
                            break;
                        }
                    }
                    if !Self::is_listen_peer_via_relay(&swarm) {
                        tracing::error!("Failed to listen on relay: {:?}", relay_addr);
                        let _ = response_tx.send(Err("Failed to listen on relay".into()));
                        return;
                    }
                }
                let _ = response_tx.send(Ok(()));
            }
            P2pCommand::ConnectToPeer {
                remote_peer_id,
                response_tx,
            } => {
                if let Ok(()) = Self::dial_peer(swarm, remote_peer_id, &relay_addr) {
                    pending_requests.insert(remote_peer_id.to_string(), response_tx);
                } else {
                    tracing::error!("Failed to dial to peer: {:?}", remote_peer_id);
                    let _ = response_tx.send(Err("Failed to dial to peer".into()));
                }
            }
            P2pCommand::RecvFile {
                remote_peer_id,
                src_path: target_path,
                tgt_path: save_path,
                response_tx,
            } => {
                tracing::info!("File recv request isstarting : {:?}", target_path);
                if PeerId::from_str(&remote_peer_id).is_err() {
                    tracing::error!("Invalid PeerId: {:?}", remote_peer_id);
                    let _ = response_tx.send(Err("Invalid PeerId".into()));
                    return;
                }
                let remote_peer_id = PeerId::from_str(&remote_peer_id).expect("Invalid PeerId");

                tracing::info!("Connecting to peer...");
                if !Self::is_peer_connected(&swarm, &remote_peer_id.to_string()) {
                    for _ in 0..MAX_DIAL_RETRY {
                        let _ = Self::dial_peer(swarm, remote_peer_id, &relay_addr);
                        if Self::is_peer_connected(&swarm, &remote_peer_id.to_string()) {
                            tracing::info!("Connected to peer");
                            break;
                        } else {
                            tracing::warn!("Failed to connect to peer: {:?}", remote_peer_id);
                        }
                    }
                    if !Self::is_peer_connected(&swarm, &remote_peer_id.to_string()) {
                        tracing::error!("Failed to connect to peer: {:?}", remote_peer_id);
                        let _ = response_tx.send(Err("Failed to connect to peer".into()));
                        return;
                    }
                }
                tracing::info!("Connected to peer");

                if let Some(file_name) = std::path::Path::new(&target_path).file_name() {
                    let file_name = file_name.to_string_lossy().to_string();
                    let save_path = save_path.clone();
                    let request = ku_protocol::FileRequest {
                        file_name: file_name.clone(),
                        target_path: target_path.clone(),
                        save_path: save_path.clone(),
                    };
                    swarm
                        .behaviour_mut()
                        .ku_file_transfer
                        .send_request(&remote_peer_id, request);

                    pending_requests.insert(target_path.clone(), response_tx);
                    tracing::info!(
                        "File recv request: {:?} is sent and listening for res",
                        target_path
                    );
                } else {
                    tracing::error!("Invalid file path: {:?}", target_path);
                    let _ = response_tx.send(Err("Invalid file path".into()));
                }
            }
            P2pCommand::SendFileOpen {
                src_path,
                response_tx,
            } => {
                tracing::info!("File send request: is starting : {:?}", src_path);
                let listen_addrs: Vec<String> =
                    swarm.listeners().map(|addr| addr.to_string()).collect();
                if !listen_addrs.iter().any(|addr| addr.contains("p2p-circuit")) {
                    tracing::error!("Swarm is not listening while SendFileOpen");
                    let _ = response_tx.send(Err("Failed to listen via relay".into()));
                    return;
                }
                pending_requests.insert(src_path.clone(), response_tx);
                tracing::info!("File send request: {:?} is listening", src_path);
            }
        }
    }

    fn is_relay_connected(swarm: &Swarm<Behaviour>, relay_addr: &str) -> bool {
        let relay_id = relay_addr.rsplit('/').next().unwrap_or("");
        tracing::info!("Relay ID: {:?}", relay_id);
        swarm.connected_peers().any(|peer_id| {
            tracing::info!(
                "Peer ID: {:?} equals relay {:?}",
                peer_id,
                (&*peer_id.to_string() == relay_id)
            );
            &*peer_id.to_string() == relay_id
        });
        swarm
            .connected_peers()
            .any(|peer_id| &*peer_id.to_string() == relay_id)
    }

    fn is_peer_connected(swarm: &Swarm<Behaviour>, remote_peer_id: &str) -> bool {
        swarm.connected_peers().for_each(|peer_id| {
            tracing::info!(
                "Peer ID: {:?} equals remote {:?}",
                peer_id,
                (&*peer_id.to_string() == remote_peer_id)
            );
        });
        swarm
            .connected_peers()
            .any(|peer_id| &(*peer_id.to_string()) == remote_peer_id)
    }

    fn is_listen_peer_via_relay(swarm: &Swarm<Behaviour>) -> bool {
        swarm
            .listeners()
            .any(|addr| addr.iter().any(|proto| proto == Protocol::P2pCircuit))
    }

    async fn dial_relay(
        swarm: &mut Swarm<Behaviour>,
        relay_address: &Multiaddr,
    ) -> Result<(), DialError> {
        swarm.dial(relay_address.clone())
    }

    async fn listen_peer_via_relay(
        swarm: &mut Swarm<Behaviour>,
        relay_addr: &Multiaddr,
    ) -> Result<(), Box<dyn Error>> {
        let relay_address = relay_addr.clone();
        swarm.listen_on(relay_address.with(Protocol::P2pCircuit))?;
        Ok(())
    }

    async fn init_swarm(
        secret_key_seed: &str,
        relay_addr: &mut Multiaddr,
    ) -> Result<Swarm<Behaviour>, Box<dyn Error>> {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .try_init();

        let mut swarm =
            libp2p::SwarmBuilder::with_existing_identity(generate_ed25519(secret_key_seed))
                .with_tokio()
                .with_tcp(
                    tcp::Config::default().nodelay(true),
                    noise::Config::new,
                    yamux::Config::default,
                )?
                // .with_quic()
                // .with_dns()?
                .with_relay_client(noise::Config::new, yamux::Config::default)?
                .with_behaviour(|keypair, relay_behaviour| Behaviour {
                    relay_client: relay_behaviour,
                    ping: ping::Behaviour::new(ping::Config::new()),
                    identify: identify::Behaviour::new(identify::Config::new(
                        "/KUDRIVE/0.0.1".to_string(),
                        keypair.public(),
                    )),
                    dcutr: dcutr::Behaviour::new(keypair.public().to_peer_id()),
                    ku_file_transfer: request_response::Behaviour::with_codec(
                        KuFileTransferCodec(),
                        vec![(
                            StreamProtocol::new("/ku-file-transfer/1.0.0"),
                            ProtocolSupport::Full,
                        )],
                        Default::default(),
                    ),
                    // ku_messaging: request_response::Behaviour::with_codec(
                    //     MessagingCodec,
                    //     vec![(
                    //         StreamProtocol::new("/ku-messaging/1.0.0"),
                    //         ProtocolSupport::Full,
                    //     )],
                    //     Default::default(),
                    // ),
                })?
                .with_swarm_config(|c|
                    // @@ TODO : config test
                    c.with_idle_connection_timeout(Duration::from_secs(SWARM_IDLE_TIMEOUT))
                    .with_per_connection_event_buffer_size(SWARM_CONN_BUF_SIZE)
                    .with_max_negotiating_inbound_streams(SWARM_MAX_NEGOTIATING_INBOUND_STREAMS)
                    .with_notify_handler_buffer_size(NonZero::new(SWARM_NOTIFY_BUF_SIZE).expect("SWARM_NOTIFY_BUF_SIZE must be NonZero"))
                )
                .build();
        // swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        block_on(async {
            let mut delay =
                futures_timer::Delay::new(std::time::Duration::from_secs(INIT_LISTEN_DELAY)).fuse();
            loop {
                futures::select! {
                    event = swarm.next() => {
                        match event.unwrap() {
                            SwarmEvent::NewListenAddr { address, .. } => {
                                tracing::info!(%address, "Listening on address");
                            }
                            event => panic!("{event:?}"),
                        }
                    }
                    _ = delay => {
                        break;
                    }
                }
            }
        });
        // swarm.listen_on(relay_addr.clone().with(Protocol::P2pCircuit))?;
        Ok(swarm)
    }

    fn dial_peer(
        swarm: &mut Swarm<Behaviour>,
        remote_peer_id: PeerId,
        relay_address: &Multiaddr,
    ) -> Result<(), DialError> {
        swarm.dial(
            relay_address
                .clone()
                .with(Protocol::P2pCircuit)
                .with(Protocol::P2p(remote_peer_id)),
        )
    }

    pub async fn handle_swarm_event(
        swarm: &mut Swarm<Behaviour>,
        relay_address: &Multiaddr,
        told_relay_observed_addr: &mut bool,
        learned_observed_addr: &mut bool,
        event: SwarmEvent<BehaviourEvent>,
        // base_dir_path: std::path::PathBuf,
        pending_requests: &mut HashMap<String, oneshot::Sender<Result<(), String>>>,
    ) -> Result<(), Box<dyn Error>> {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                tracing::info!(%address, "Listening on address");
            }
            SwarmEvent::Behaviour(BehaviourEvent::RelayClient(
                relay::client::Event::ReservationReqAccepted { .. },
            )) => {
                tracing::info!("Relay accepted our reservation request");
            }
            SwarmEvent::Behaviour(BehaviourEvent::RelayClient(event)) => {
                tracing::info!(?event)
            }
            SwarmEvent::Behaviour(BehaviourEvent::Dcutr(event)) => {
                tracing::info!(?event)
            }
            SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Sent { .. })) => {
                tracing::info!("Told relay its public address");
                *told_relay_observed_addr = true;
                if *told_relay_observed_addr && *learned_observed_addr {
                    if let Some(sender) = pending_requests.remove(&relay_address.to_string()) {
                        let _ = sender.send(Ok(()));
                    }
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received {
                info: identify::Info { observed_addr, .. },
                ..
            })) => {
                tracing::info!(address=%observed_addr, "Relay told us our observed address");
                *learned_observed_addr = true;
                if *told_relay_observed_addr && *learned_observed_addr {
                    if let Some(sender) = pending_requests.remove(&relay_address.to_string()) {
                        let _ = sender.send(Ok(()));
                    }
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Ping(_)) => {}
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                if let Some(sender) = pending_requests.remove(&peer_id.to_string()) {
                    tracing::info!(peer=%peer_id, ?endpoint, "Established new connection!!!");
                    let _ = sender.send(Ok(()));
                }
                tracing::info!(peer=%peer_id, ?endpoint, "Established new connection");
            }
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                if let Some(peer_id) = peer_id {
                    if let Some(sender) = pending_requests.remove(&peer_id.to_string()) {
                        let _ = sender.send(Err(error.to_string()));
                    }
                    tracing::info!(peer=%peer_id, "Outgoing connection failed: {error}");
                }
            }
            // SwarmEvent::Behaviour(BehaviourEvent::KuMessaging(event)) => {
            //     println!(">>{:?}", event);
            //     match event {
            //         request_response::Event::Message { peer: _, message } => {
            //             match message {
            //                 request_response::Message::Request { request, channel, .. } => {
            //                     println!("Received request: {:?}", request);
            //                     let response = KuMessage(format!("Echo: {}", request.0));
            //                     swarm
            //                         .behaviour_mut()
            //                         .ku_messaging
            //                         .send_response(channel, response)
            //                         .expect("Failed to send response");
            //                 }
            //                 request_response::Message::Response { request_id, response } => {
            //                     println!("Received response to request {:?}: {:?}", request_id, response);
            //                 }
            //             }
            //         }
            //         request_response::Event::OutboundFailure { peer, request_id, error } => {
            //             println!("Outbound failure for peer {:?}, request {:?}: {:?}", peer, request_id, error);
            //         }
            //         request_response::Event::InboundFailure { peer, request_id, error } => {
            //             println!("Inbound failure for peer {:?}, request {:?}: {:?}", peer, request_id, error);
            //         }
            //         request_response::Event::ResponseSent { peer, request_id } => {
            //             println!("Response sent to peer {:?}, request {:?}", peer, request_id);
            //         }
            //     }
            // }
            SwarmEvent::Behaviour(BehaviourEvent::KuFileTransfer(event)) => match event {
                request_response::Event::Message { peer, message } => match message {
                    request_response::Message::Request {
                        request, channel, ..
                    } => {
                        tracing::info!("Received file request: {:?}", request);
                        let sender_opt: Option<oneshot::Sender<Result<(), String>>>;
                        if let Some(sender) = pending_requests.remove(&request.target_path) {
                            sender_opt = Some(sender);
                        } else {
                            tracing::warn!(
                                "No pending request found for response file: {}",
                                request.file_name
                            );
                            sender_opt = None;
                        }
                        let file_path = std::path::Path::new(&request.target_path);
                        if let Ok(mut file) = File::open(file_path).await {
                            let mut content = Vec::new();
                            if file.read_to_end(&mut content).await.is_ok() {
                                let response = ku_protocol::FileResponse {
                                    file_name: request.file_name.clone(),
                                    content,
                                    src_path: request.target_path,
                                    tgt_path: request.save_path,
                                };
                                swarm
                                    .behaviour_mut()
                                    .ku_file_transfer
                                    .send_response(channel, response)
                                    .expect("Failed to send file response");
                                if let Some(sender) = sender_opt {
                                    let _ = sender.send(Ok(()));
                                }
                                tracing::info!("Sent file: {}", request.file_name);
                            } else {
                                let error_response = ku_protocol::FileResponse {
                                    file_name: "KUDrive Error: File read error".to_string(),
                                    content: Vec::new(),
                                    src_path: request.target_path.clone(),
                                    tgt_path: request.save_path,
                                };
                                swarm
                                    .behaviour_mut()
                                    .ku_file_transfer
                                    .send_response(channel, error_response)
                                    .expect("Failed to send error response");
                                if let Some(sender) = sender_opt {
                                    let _ = sender.send(Err(format!(
                                        "Failed to read file after open: {}",
                                        request.target_path
                                    )));
                                }
                                tracing::error!(
                                    "Failed to read file after open: {}",
                                    request.file_name
                                );
                            }
                        } else {
                            let error_response: ku_protocol::FileResponse =
                                ku_protocol::FileResponse {
                                    file_name: "KUDrive Error: File Not Found".to_string(),
                                    content: Vec::new(),
                                    src_path: request.target_path.clone(),
                                    tgt_path: request.save_path,
                                };
                            swarm
                                .behaviour_mut()
                                .ku_file_transfer
                                .send_response(channel, error_response)
                                .expect("Failed to send error response");
                            if let Some(sender) = sender_opt {
                                let _ = sender.send(Err(format!(
                                    "Failed to open file: {}",
                                    request.target_path
                                )));
                            }
                            tracing::error!("File not found: {}", request.file_name);
                        }
                    }
                    request_response::Message::Response { response, .. } => {
                        tracing::info!(
                            "File Response recieved: {:?} > {:?} ",
                            &response.src_path,
                            &response.tgt_path
                        );
                        let sender_opt: Option<oneshot::Sender<Result<(), String>>>;
                        if let Some(sender) = pending_requests.remove(&response.src_path) {
                            sender_opt = Some(sender);
                        } else {
                            tracing::warn!(
                                "No pending request found for response file: {}",
                                response.file_name
                            );
                            sender_opt = None;
                        }
                        if response.file_name.starts_with("KUDrive Error:") {
                            tracing::error!(
                                "Error occurred while receiving the file: {}",
                                response.file_name
                            );
                            if let Some(sender) = sender_opt {
                                let _ = sender.send(Err(response.file_name));
                            }
                        } else {
                            tracing::info!("Received file from {:?}: {}", peer, response.file_name);
                            // let file_path = base_dir_path.join(&response.file_name);
                            if let Ok(mut file_path) = PathBuf::from_str(&response.tgt_path) {
                                // file_path = file_path.join(&response.file_name);
                                // if tokio::fs::write(&file_path, &response.content)
                                if tokio::fs::write(&file_path, &response.content)
                                    .await
                                    .is_ok()
                                {
                                    tracing::info!("File saved to: {:?}", file_path);
                                    if let Some(sender) = sender_opt {
                                        let _ = sender.send(Ok(()));
                                    }
                                } else {
                                    tracing::error!("Failed to save file to: {:?}", file_path);
                                    if let Some(sender) = sender_opt {
                                        let _ = sender.send(Err(format!(
                                            "Failed to save file: {}",
                                            response.file_name
                                        )));
                                    }
                                }
                            } else {
                                if let Some(sender) = sender_opt {
                                    let _ = sender.send(Err(format!(
                                        "Invalid target path: {}",
                                        response.tgt_path
                                    )));
                                }
                            }
                        }
                    }
                },
                request_response::Event::OutboundFailure {
                    peer,
                    request_id,
                    error,
                } => {
                    tracing::error!(
                        peer=%peer, request_id=?request_id,
                        "Outbound failure occurred: {:?}", error
                    );
                }
                request_response::Event::InboundFailure {
                    peer,
                    request_id,
                    error,
                } => {
                    tracing::error!(
                        peer=%peer, request_id=?request_id,
                        "Inbound failure occurred: {:?}", error
                    );
                }
                request_response::Event::ResponseSent { peer, request_id } => {
                    tracing::info!(peer=%peer, request_id=?request_id, "Response sent successfully");
                }
            },
            _ => {}
        }
        Result::Ok(())
    }
}

pub async fn run_cli_command(
    client: &mut P2PTransport,
    cmd: &str,
    send_rx: &mut Option<oneshot::Receiver<Result<(), String>>>,
) -> bool {
    match cmd {
        "exit" => {
            println!(">>{:?}", "Exiting...");
            cli_helpfn();
            // break;
            return true;
        }
        "l" => {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let res = client
                .command_tx
                .send(P2pCommand::GetId { response_tx: tx })
                .await;
            if res.is_err() {
                println!("Error: {:?}", res);
            }
            let res = rx.await;
            match res {
                Ok(peer_id) => println!("Local peer id: {:?}", peer_id),
                Err(e) => eprintln!("Failed to get local peer id: {}", e),
            }
            cli_helpfn();
        }
        "p" => {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let _ = client
                .command_tx
                .send(P2pCommand::GetStatus { response_tx: tx })
                .await
                .expect("Failed to send command");
            let res = rx.await;
            match res {
                Ok(status) => match status {
                    P2pStatus::NotConnected => println!("Status : Not connected"),
                    P2pStatus::RelayConnected => println!("Status : Relay connected"),
                    P2pStatus::PeerConnected(peers) => {
                        println!("Status : Peer connected");
                        for peer in peers {
                            println!("  - {:?}", peer);
                        }
                    }
                },
                Err(e) => eprintln!("Failed to get status: {}", e),
            }
            cli_helpfn();
        }
        "ls" => {
            match std::fs::read_dir(".") {
                Ok(entries) => {
                    println!("Files in current directory:");
                    for entry in entries {
                        if let Ok(entry) = entry {
                            println!("  - {:?}", entry.file_name());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to list directory: {}", e);
                }
            }
            cli_helpfn();
        }
        "relay" => {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let _ = client
                .command_tx
                .send(P2pCommand::ConnectToRelay { response_tx: tx })
                .await;
            tokio::select! {
                res = rx => match res {
                    Ok(res) => {
                        println!("Relay connected: {:?}", res);
                    }
                    Err(e) => eprintln!("Failed to get pending requests: {}", e),
                },
                _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {
                    println!(">>{:?}", "Aborted")
                }
            }
            cli_helpfn();
        }
        "req" => {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let _ = client
                .command_tx
                .send(P2pCommand::GetPendingRequests { response_tx: tx })
                .await;
            tokio::select! {
                res = rx => match res {
                    Ok(requests) => {
                        println!("Pending requests:");
                        for req in requests {
                            println!("  - {:?}", req);
                        }
                    }
                    Err(e) => eprintln!("Failed to get pending requests: {}", e),
                },
                _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {
                    println!(">>{:?}", "Aborted")
                }
            }
            cli_helpfn();
        }
        "" => {}
        "lis" => {
            let res = client.listen_on_peer(5).await;
            match res {
                Ok(_) => println!("Listening on peer"),
                Err(e) => eprintln!("Failed to listen on peer: {}", e),
            }
        }
        "liss" => {
            let res = client.get_listen_addr(5).await;
            match res {
                Ok(addrs) => {
                    println!("Listening on peer ({})", addrs.len());
                    for addr in addrs {
                        println!("  - {:?}", addr);
                    }
                }
                Err(e) => eprintln!("Failed to listen on peer: {}", e),
            }
        }
        "lis_check" => {
            let res = client.is_listening(5).await;
            match res {
                Ok(true) => println!("Listening on peer"),
                Ok(false) => println!("Not listening on peer"),
                Err(e) => eprintln!("Failed to listen on peer: {}", e),
            }
        }
        other if other.starts_with("dial-") => {
            let parts: Vec<&str> = other.splitn(2, '-').collect();
            if parts.len() >= 2 {
                let peer_id = parts[1].to_string();
                let (tx, rx) = tokio::sync::oneshot::channel();
                let _ = client
                    .command_tx
                    .send(P2pCommand::ConnectToPeer {
                        remote_peer_id: PeerId::from_str(&peer_id)
                            .expect("Error parsing str to peer_id"),
                        response_tx: tx,
                    })
                    .await;
                let res = rx.await;
                match res {
                    Ok(res) => match res {
                        Ok(_) => println!("Connected to peer {}", peer_id),
                        Err(e) => eprintln!("Failed to dial peer {}: {}", peer_id, e),
                    },
                    Err(e) => eprintln!("Failed to dial peer {}: {}", peer_id, e),
                }
            } else {
                eprintln!("Invalid format. Use 'dial-<peer_id>'");
            }
            cli_helpfn();
        }
        other if other.starts_with("r-") => {
            let parts: Vec<&str> = other.splitn(4, '-').collect();
            if parts.len() >= 4 {
                let remote_peer_id = parts[1].to_string();
                let target_path = parts[2].to_string();
                let save_path = parts[3].to_string();
                tokio::select! {
                    res = client.recv_file(remote_peer_id, target_path, save_path, 10) => {
                        match res {
                            Ok(_) => println!("File received successfully."),
                            Err(e) => eprintln!("Failed to receive file: {:?}", e)
                        }
                    }
                    _ = tokio::time::sleep(std::time::Duration::from_secs(20)) => {
                        return false;
                    }
                    _ = tokio::signal::ctrl_c() => {
                        println!(">>{:?}", "Aborted");
                        return true;
                    }
                }
            } else {
                eprintln!("Invalid format. Use 'r-<remote_peer_id>-<target_path>-<save_path>'");
            }
            cli_helpfn();
        }
        other if other.starts_with("os-") => {
            let parts: Vec<&str> = other.splitn(2, '-').collect();
            if parts.len() >= 2 {
                let src_path = parts[1].to_string();
                tokio::select! {
                    _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {}
                    _ = tokio::signal::ctrl_c() => {
                        println!(">>{:?}", "Aborted");
                        return true;
                    }
                    res = client.send_file_open(src_path) => {
                        match res {
                            Ok(rx) => {
                                *send_rx = Some(rx);
                                println!("File sent open successfully.")},
                            Err(e) => eprintln!("Failed to send file: {}", e),
                        }
                    }
                }
            } else {
                eprintln!("Invalid format. Use 's-<remote_peer_id>-<target_path>'");
            }
            cli_helpfn();
        }
        other if other.starts_with("ws") => {
            if let Some(rx) = send_rx {
                tokio::select! {
                    _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {}
                    _ = tokio::signal::ctrl_c() => {
                        println!(">>{:?}", "Aborted");
                        return true;
                    }
                    res = client.send_file_wait(rx, 30)=> {
                        match res {
                            Ok(_) => println!("File sent successfully."),
                            Err(e) => eprintln!("Failed to send file: {}", e),
                        }
                    }
                }
            } else {
                eprintln!("No reciever given");
            }
            cli_helpfn();
        }
        _ => {
            println!("Unknown command");
            cli_helpfn();
        }
    }
    false
}

pub fn cli_helpfn() {
    println!(
        r#"
Commands:
- 'exit' to exit
- 'l' to list local peer id
- 'p' to list connection status & peers
- 'ls' to list files in the current directory
- 'relay' to connect to relay
- 'req' to list pending requests
- 'lis' to listen on peer
- 'dial-<peer_id>' to dial a peer
- 'r-<remote_peer_id>-<target_path>-<save_path>' to receive a file
- 'os-<src_path>' to send a file open
- 'ws' to send a file wait"#
    )
}
