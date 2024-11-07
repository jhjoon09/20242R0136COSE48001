use std::net::Ipv4Addr;

pub struct Transport {
    server_addr: Ipv4Addr,
}

impl Transport {
    pub fn new() -> Self {
        Self {
            server_addr: Ipv4Addr::new(127, 0, 0, 1),
        }
    }

    pub async fn connect(&self) {
        println!("Connecting to server...");
    }

    pub async fn disconnect(&self) {
        println!("Disconnecting from server...");
    }
}
