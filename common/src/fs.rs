use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMap {
    pub files: Vec<File>,
}
