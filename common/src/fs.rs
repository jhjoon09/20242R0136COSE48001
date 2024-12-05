use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMap {
    pub files: Vec<File>,
}
