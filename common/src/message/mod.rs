pub mod client;
pub mod server;

pub trait Message: Send + 'static {
    fn from_bytes(bytes: &[u8]) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
}
