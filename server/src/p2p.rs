use futures::StreamExt;
use kudrive_common::p2p::generate_ed25519;
use libp2p::{
    core::{multiaddr::Protocol, Multiaddr},
    dcutr, identify, identity, noise, ping, relay,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use std::error::Error;
use std::net::Ipv4Addr;
use tokio::{
    io::{self, AsyncBufReadExt as _},
    select,
    sync::mpsc,
};
use tracing_subscriber::EnvFilter;

const P2P_ID: &str = "0";
const PORT: u16 = 4001;

pub enum P2PCommand {
    Exit,
    StrMsg(String),
}

pub async fn p2p_test_run() {
    let (tx, rx) = tokio::sync::mpsc::channel(32);
    tokio::task::spawn(async {
        let _ = p2p_relay_run(rx, 4001).await;
    });
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    loop {
        tokio::select! {
            Ok(Some(line)) = stdin.next_line() => {
                match line.as_str() {
                    "exit" => {
                        tx.send(P2PCommand::Exit).await.unwrap();
                        break;
                    }
                    "" => {}
                    _ => {
                        tx.send(P2PCommand::StrMsg(line.clone())).await.unwrap();
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Ctrl+C input Returning...");
                return;
            }
        }
    }
}

pub async fn p2p_relay_run(
    mut event_msg: mpsc::Receiver<P2PCommand>,
    port: u16,
) -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let local_key: identity::Keypair = generate_ed25519(P2P_ID);

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
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

    // let listen_addr_quic = Multiaddr::empty()
    //     .with(Protocol::from(Ipv4Addr::UNSPECIFIED))
    //     .with(Protocol::Udp(port))
    //     .with(Protocol::QuicV1);
    // swarm.listen_on(listen_addr_quic.clone())?;

    loop {
        select! {
            Some(swarm_event) = swarm.next() => {
                match swarm_event {
                    SwarmEvent::Behaviour(event) => {
                        if let BehaviourEvent::Identify(identify::Event::Received {
                            info: identify::Info { observed_addr, .. },
                            ..
                        }) = &event
                        {
                            swarm.add_external_address(observed_addr.clone());
                        }
                        println!("{event:?}")
                    }
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {address:?}");
                    }
                    _ => {}
                }
            },
            Some(msg) = event_msg.recv() => {
                match msg {
                    P2PCommand::Exit =>{
                        println!("main-thread invoked EXIT");
                        break;
                    }
                    P2PCommand::StrMsg(msg) => {
                        println!("from main-thread: \"{}\"", msg);
                        if msg == "info" {
                            println!("external_addresses:");
                            for a in swarm.external_addresses() {
                                println!("{:?}", a);
                            }
                            println!("relay listen address: \"{}\"", listen_addr_tcp);
                            println!("local peer id : {:?}",swarm.local_peer_id() );
                        } else {}
                    }
                }
            }
        }
    }
    Ok(())
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    relay: relay::Behaviour,
    ping: ping::Behaviour,
    identify: identify::Behaviour,
    dcutr: dcutr::Behaviour,
}
