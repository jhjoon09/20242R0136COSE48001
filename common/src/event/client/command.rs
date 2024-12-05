use crate::Client;

#[derive(Debug)]

pub enum Command {
    Clients {},
}

#[derive(Debug)]
pub enum Consequence {
    Clients { result: Result<Vec<Client>, String> },
}
