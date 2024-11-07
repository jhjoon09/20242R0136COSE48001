use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirTree {
    files: Vec<FileMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    name: String,
    size: u64,
    modified: u64,
}
