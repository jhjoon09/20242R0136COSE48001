use crate::constant::EventMsg;
use clap::Parser;
use futures::StreamExt;
use kudrive_common::p2p::generate_ed25519;
use libp2p::{
    core::{multiaddr::Protocol, Multiaddr},
    dcutr, identify, identity, noise, ping, relay,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use std::error::Error;
use std::net::{Ipv4Addr, Ipv6Addr};
use tokio::{select, sync::mpsc};
use tracing_subscriber::EnvFilter;

const P2P_ID: &str = "KudriveRelay";

pub async fn p2p_relay_run(mut event_msg: mpsc::Receiver<EventMsg>) -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let opt = Opt::parse();

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
        .with(match opt.use_ipv6 {
            Some(true) => Protocol::from(Ipv6Addr::UNSPECIFIED),
            _ => Protocol::from(Ipv4Addr::UNSPECIFIED),
        })
        .with(Protocol::Tcp(opt.port));
    swarm.listen_on(listen_addr_tcp.clone())?;

    let listen_addr_quic = Multiaddr::empty()
        .with(match opt.use_ipv6 {
            Some(true) => Protocol::from(Ipv6Addr::UNSPECIFIED),
            _ => Protocol::from(Ipv4Addr::UNSPECIFIED),
        })
        .with(Protocol::Udp(opt.port))
        .with(Protocol::QuicV1);
    swarm.listen_on(listen_addr_quic.clone())?;

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
                    EventMsg::EXIT =>{
                        println!("main-thread invoked EXIT");
                        break;
                    }
                    EventMsg::STR_MSG(msg) => {
                        println!("from main-thread: \"{}\"", msg);
                        if msg == "1" {
                            println!("external_addresses:");
                            for a in swarm.external_addresses() {
                                println!("{:?}", a);
                            }
                            println!("relay listen address: \n\t-\"{}\"\n\t-\"{}\"", listen_addr_tcp, listen_addr_quic);
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

#[derive(Debug, Parser)]
#[clap(name = "libp2p relay")]
struct Opt {
    /// Determine if the relay listen on ipv6 or ipv4 loopback address. the default is ipv4
    #[clap(long)]
    use_ipv6: Option<bool>,

    /// Fixed value to generate deterministic peer id
    #[clap(long)]
    secret_key_seed: u8,

    /// The port used to listen on all interfaces
    #[clap(long)]
    port: u16,
}
