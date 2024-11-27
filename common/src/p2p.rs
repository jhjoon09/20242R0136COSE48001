use libp2p::{identity, Multiaddr, PeerId};
use serde::{Deserialize, Serialize};
use crate::{ClientInfo, FileMetadata};


pub fn generate_ed25519(secret_key_seed: &str) -> identity::Keypair {
    let mut bytes = [0u8; 32];
    let seed_bytes = secret_key_seed.as_bytes();
    let len = seed_bytes.len().min(32);
    bytes[..len].copy_from_slice(&seed_bytes[..len]);

    identity::Keypair::ed25519_from_bytes(bytes).expect("only errors on wrong length")
}
