use lazy_static::lazy_static;
use serde::Deserialize;
use std::fs;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub file: FileConfig,
    pub id: IdConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub domain: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct FileConfig {
    pub workspace: String,
    pub refresh_time: u32,
    pub ignore_list: Vec<String>, // ignore_list를 Vec<String>으로 매핑
}

#[derive(Debug, Deserialize)]
pub struct IdConfig {
    pub group_id: Uuid,
    pub my_id: Uuid,
    pub nickname: String,
}

lazy_static! {
    static ref CONFIG: Config = {
        let file_path = "../config/client.yaml"; // YAML 파일 경로
        let contents = fs::read_to_string(file_path)
            .expect("Failed to read config file");
        serde_yaml::from_str(&contents)
            .expect("Failed to parse config file")
    };
}

pub fn get_config() -> &'static Config {
    &CONFIG
}

