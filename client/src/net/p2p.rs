use crate::net::ku_protocol;
use clap::Parser;
use futures::{executor::block_on, future::FutureExt, stream::StreamExt};
use libp2p::{
    core::multiaddr::{Multiaddr, Protocol},
    dcutr, identify, noise, ping, relay,
    request_response::{self, ProtocolSupport},
    swarm::{DialError, NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, StreamProtocol, Swarm,
};
use std::str::FromStr;
use std::{error::Error, time::Duration};
use tokio::{
    fs::File,
    io::{self, AsyncBufReadExt, AsyncReadExt},
};
use tracing_subscriber::EnvFilter;
use kudrive_common::p2p::generate_ed25519;
use super::ku_protocol::KuFileTransferCodec;

const MAX_DIAL_RETRY: usize = 3;
const INIT_LISTEN_DELAY: u64 = 2;

#[derive(Debug, Parser)]
#[clap(name = "libp2p DCUtR client")]
struct Opts {
    /// The mode (client-listen, client-dial).
    #[clap(long)]
    mode: P2pMode,

    /// Fixed value to generate deterministic peer id.
    #[clap(long)]
    secret_key_seed: String,

    /// The listening address
    #[clap(long)]
    relay_address: Multiaddr,

    /// Peer ID of the remote peer to hole punch to.
    #[clap(long)]
    remote_peer_id: Option<PeerId>,

    #[clap(long, default_value = "./received_files")]
    path: String,
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum P2pMode {
    Dial,
    Listen,
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


pub async fn run_client() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();
    let mut swarm = init(&opts.relay_address, &opts.secret_key_seed).await?;

    let mode = opts.mode;
    let relay_address = opts.relay_address;
    let relay_address_clone = relay_address.clone();

    println!("Running client in {:?} mode", mode);
    println!("local peer id : {:?}", swarm.local_peer_id());
    match mode {
        P2pMode::Dial => {
            let remote_peer_id = opts.remote_peer_id.unwrap();
            dial_peer(&mut swarm, &relay_address, remote_peer_id)?;
        }
        P2pMode::Listen => {
            swarm.listen_on(relay_address.with(Protocol::P2pCircuit))?;
        }
    }
    let mut remote_peer_id = None;
    if mode == P2pMode::Dial {
        remote_peer_id = Some(opts.remote_peer_id.unwrap());
    }
    handle_swarm_event(&mut swarm, mode, &relay_address_clone, remote_peer_id).await
}

async fn get_file(
    swarm: &mut Swarm<Behaviour>,
    remote_peer_id: PeerId,
    file_path: String,
) -> Result<(), Box<dyn Error>> {
    // Create a FileRequest
    let request = ku_protocol::FileRequest {
        file_name: file_path.clone(),
    };
    if swarm.connected_peers().all(|peer| *peer != remote_peer_id) {
        for _ in 0..MAX_DIAL_RETRY {
            if dial_peer(swarm, &Multiaddr::empty(), remote_peer_id).is_ok() {
                break;
            }
        }
    }
    // Send the file request to the remote peer
    swarm
        .behaviour_mut()
        .ku_file_transfer
        .send_request(&remote_peer_id, request);

    println!("Sent file request for: {}", file_path);
    Ok(())
}

pub async fn init(
    relay_address: &Multiaddr,
    secret_key_seed: &str,
) -> Result<Swarm<Behaviour>, Box<dyn Error>> {
    let mut swarm = init_swarm(secret_key_seed).await?;
    init_connect_swarm(&mut swarm, relay_address).await?;
    Ok(swarm)
}

pub async fn init_swarm(secret_key_seed: &str) -> Result<Swarm<Behaviour>, Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        // .with_max_level(filter::LevelFilter::DEBUG)
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let swarm = libp2p::SwarmBuilder::with_existing_identity(generate_ed25519(secret_key_seed))
        .with_tokio()
        .with_tcp(
            tcp::Config::default().nodelay(true),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_dns()?
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
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();
    Ok(swarm)
}

pub async fn init_connect_swarm(
    swarm: &mut Swarm<Behaviour>,
    relay_address: &Multiaddr,
) -> Result<(), Box<dyn Error>> {
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
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

    swarm.dial(relay_address.clone()).unwrap();
    block_on(async {
        let mut learned_observed_addr = false;
        let mut told_relay_observed_addr = false;
        swarm.connected_peers().for_each(|peer_id| {
            tracing::info!(peer=%peer_id, "Connected to relay");
        });
        loop {
            match swarm.next().await.unwrap() {
                SwarmEvent::NewListenAddr { .. } => {}
                SwarmEvent::Dialing { .. } => {}
                SwarmEvent::ConnectionEstablished { .. } => {}
                SwarmEvent::Behaviour(BehaviourEvent::Ping(_)) => {}
                SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Sent {
                    ..
                })) => {
                    tracing::info!("Told relay its public address");
                    told_relay_observed_addr = true;
                }
                SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received {
                    info: identify::Info { observed_addr, .. },
                    ..
                })) => {
                    tracing::info!(address=%observed_addr, "Relay told us our observed address");
                    learned_observed_addr = true;
                }
                event => panic!("PANIC:```\n{event:?}\n```\n"),
            }

            if learned_observed_addr && told_relay_observed_addr {
                break;
            }
        }
    });
    Ok(())
}

pub async fn handle_swarm_event(
    swarm: &mut Swarm<Behaviour>,
    mode: P2pMode,
    relay_address: &Multiaddr,
    remote_peer_id: Option<PeerId>,
) -> Result<(), Box<dyn Error>> {
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    let opts = Opts::parse();
    let base_path = std::path::Path::new(&opts.path);
    if !base_path.exists() {
        tokio::fs::create_dir_all(&base_path).await?;
    }
    let peer_dir = base_path.join(swarm.local_peer_id().to_string());
    if !peer_dir.exists() {
        tokio::fs::create_dir_all(&peer_dir).await?;
    }
    block_on(async {
        loop {
            tokio::select! {
                Ok(Some(line)) = stdin.next_line() => {
                    match line.as_str() {
                        "exit" => {
                            println!(">>{:?}", "Exiting...");
                            break;
                        },
                        "l" => {
                            println!(">>{:?} : {}", "Local peer id:", swarm.local_peer_id());
                            swarm.external_addresses().for_each(|addr| {
                                println!("  -{:?}", addr);
                            });
                            println!("-----------")
                        },
                        "r" => {
                            if let Some(remote_peer_id) = remote_peer_id {
                                println!(">>Retrying dial...{:?}", remote_peer_id);
                                let _ = dial_peer(swarm, relay_address, remote_peer_id);
                                swarm.external_addresses().for_each(|addr| {
                                    println!("  -{:?}", addr);
                                });
                            }else{
                                println!(">>{:?}", "Remote peer id is not set");
                                continue;
                            }
                        },
                        "p" => {
                            println!("Connected peers:");
                            for peer in swarm.connected_peers() {
                                println!("  - {:?}", peer);
                            }
                        },
                        "ls"=>{
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
                        },
                        "" => {},
                        other => {
                            if let Some(peer_id) = remote_peer_id {
                                let path = std::path::Path::new(other);
                                if path.exists() && path.is_file() {
                                    // Send file if it exists
                                    match get_file(swarm, peer_id, other.to_string()).await {
                                        Ok(_) => println!("File '{}' sent successfully.", other),
                                        Err(e) => eprintln!("Failed to send file '{}': {}", other, e),
                                    }
                                } else {
                                    // let request = KuMessage(other.to_string());
                                    // swarm
                                    //     .behaviour_mut()
                                    //     .ku_messaging
                                    //     .send_request(&peer_id, request);
                                }
                            }
                        }
                    }
                    println!(r#"
    Commands:
    - 'exit' to exit
    - 'l' to list local peer id and external addresses
    - 'r' to retry dialing the remote peer
    - 'ls' to list files in the current directory
    - '<file_path>' to send a file
    - '<message>' to send a message if not file found 
                    "#);
                },
                Some(event) = swarm.next() => {
                    match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            tracing::info!(%address, "Listening on address");
                        }
                        SwarmEvent::Behaviour(BehaviourEvent::RelayClient(
                            relay::client::Event::ReservationReqAccepted { .. },
                        )) => {
                            assert!(mode == P2pMode::Listen);
                            tracing::info!("Relay accepted our reservation request");
                        }
                        SwarmEvent::Behaviour(BehaviourEvent::RelayClient(event)) => {
                            tracing::info!(?event)
                        }
                        SwarmEvent::Behaviour(BehaviourEvent::Dcutr(event)) => {
                            tracing::info!(?event)
                        }
                        SwarmEvent::Behaviour(BehaviourEvent::Identify(event)) => {
                            tracing::info!(?event)
                        }
                        SwarmEvent::Behaviour(BehaviourEvent::Ping(_)) => {}
                        SwarmEvent::ConnectionEstablished {
                            peer_id, endpoint, ..
                        } => {
                            tracing::info!(peer=%peer_id, ?endpoint, "Established new connection");
                        }
                        SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                            tracing::info!(peer=?peer_id, "Outgoing connection failed: {error}");
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
                        SwarmEvent::Behaviour(BehaviourEvent::KuFileTransfer(event)) => {
                            match event {
                                request_response::Event::Message { peer, message } => {
                                    match message {
                                        request_response::Message::Request { request, channel, .. } => {
                                            println!("Received file request: {:?}", request);
                                            let file_path = std::path::Path::new(&request.file_name);
                                            if let Ok(mut file) = File::open(file_path).await {
                                                let mut content = Vec::new();
                                                let file_name = file_path.file_name().unwrap().to_str().unwrap().to_string();
                                                if file.read_to_end(&mut content).await.is_ok() {
                                                    let response = ku_protocol::FileResponse {
                                                        // file_name: request.file_name.clone(),
                                                        file_name: file_name,
                                                        content,
                                                    };
                                                    swarm.behaviour_mut()
                                                        .ku_file_transfer
                                                        .send_response(channel, response)
                                                        .expect("Failed to send file response");
                                                    println!("Sent file: {}", request.file_name);
                                                } else {
                                                    let error_response = ku_protocol::FileResponse {
                                                        file_name: "KUDrive Error: File read error".to_string(),
                                                        content: Vec::new(),
                                                    };
                                                    swarm.behaviour_mut()
                                                        .ku_file_transfer
                                                        .send_response(channel, error_response)
                                                        .expect("Failed to send error response");
                                                    eprintln!("Failed to read file: {}", request.file_name);
                                                }
                                            } else {
                                                let error_response = ku_protocol::FileResponse {
                                                    file_name: "KUDrive Error: File Not Found".to_string(),
                                                    content: Vec::new(),
                                                };
                                                swarm.behaviour_mut()
                                                    .ku_file_transfer
                                                    .send_response(channel, error_response)
                                                    .expect("Failed to send error response");
                                                eprintln!("File not found: {}", request.file_name);
                                            }
                                        }
                                        request_response::Message::Response { response, .. } => {
                                            if response.file_name.starts_with("KUDrive Error:") {
                                                eprintln!("Error occurred while receiving the file. \n\t{}", response.file_name);
                                            } else {
                                                println!("Received file from {:?}: {}", peer, response.file_name);
                                                let file_path = peer_dir.join(&response.file_name);
                                                if tokio::fs::write(&file_path, &response.content).await.is_ok() {
                                                    println!("File saved to: {:?}", file_path);
                                                } else {
                                                    eprintln!("Failed to save file to: {:?}", file_path);
                                                }
                                            }
                                        }
                                    }
                                }
                                request_response::Event::OutboundFailure { peer, request_id, error } => {
                                    println!("Outbound failure for peer {:?}, request {:?}: {:?}", peer, request_id, error);
                                }
                                request_response::Event::InboundFailure { peer, request_id, error } => {
                                    println!("Inbound failure for peer {:?}, request {:?}: {:?}", peer, request_id, error);
                                }
                                request_response::Event::ResponseSent { peer, request_id } => {
                                    println!("Response sent to peer {:?}, request {:?}", peer, request_id);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    });
    Ok(())
}

pub fn dial_peer(
    swarm: &mut Swarm<Behaviour>,
    relay_address: &Multiaddr,
    remote_peer_id: PeerId,
) -> Result<(), DialError> {
    swarm.dial(
        relay_address
            .clone()
            .with(Protocol::P2pCircuit)
            .with(Protocol::P2p(remote_peer_id)),
    )
}

impl FromStr for P2pMode {
    type Err = String;
    fn from_str(mode: &str) -> Result<Self, Self::Err> {
        match mode {
            "dial" => Ok(P2pMode::Dial),
            "listen" => Ok(P2pMode::Listen),
            _ => Err("Expected either 'dial' or 'listen'".to_string()),
        }
    }
}