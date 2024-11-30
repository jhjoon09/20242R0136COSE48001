use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMap {
    files: Vec<File>,
}
