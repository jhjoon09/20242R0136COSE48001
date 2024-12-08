use kudrive_common::{Client, Peer};

#[derive(Debug)]

pub enum Command {
    Clients {},
    FileSend { peer: Peer },
    FileReceive { peer: Peer },
}

#[derive(Debug)]
pub enum Consequence {
    Clients { result: Result<Vec<Client>, String> },
    FileSend { result: Result<(), String> },
    FileReceive { result: Result<(), String> },
}
