use crate::{Client, Peer};

#[derive(Debug)]

pub enum Command {
    FindPeer {
        target: String,
    },
    FileSend {
        peer: Peer,
        from: String,
        to: String,
    },
    FileReceive {
        peer: Peer,
        from: String,
        to: String,
    },
    Clients {},
}

#[derive(Debug)]
pub enum Consequence {
    FindPeer { result: Result<Peer, String> },
    FileSend { result: Result<(), String> },
    FileReceive { result: Result<(), String> },
    Clients { result: Result<Vec<Client>, String> },
}
