#[derive(Debug)]

pub enum Command {
    FileSend {
        target: String,
        from: String,
        to: String,
    },
    FileReceive {
        target: String,
        from: String,
        to: String,
    },
    Clients {},
}

#[derive(Debug)]
pub enum Consequence {
    FileSend { result: Result<(), String> },
    FileReceive { result: Result<(), String> },
    Clients { result: Result<(), String> },
}
