pub mod client;
mod file;
pub mod server;

pub use file::FileClaim;

pub trait Message {
    fn from_bytes(bytes: &[u8]) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
}
