use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OS {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMap {
    pub os : OS,
    pub files: Vec<File>,
    pub folders: Vec<Folder>,
}
