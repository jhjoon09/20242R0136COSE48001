use lazy_static::lazy_static;
use serde::{ Deserialize, Serialize };
use std::{fs, io::Write};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub file: FileConfig,
    pub id: IdConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub domain: String,
    pub port: u16,
    pub p2p_relay_addr: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileConfig {
    pub workspace: String,
    pub refresh_time: u32,
    pub ignore_list: Vec<String>, // ignore_list를 Vec<String>으로 매핑
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IdConfig {
    pub group_id: Uuid,
    pub my_id: Uuid,
    pub nickname: String,
}

const CONFIG_FILE_PATH: &str = "./client.yaml";

lazy_static! {
    static ref CONFIG: Config = {
        let contents = fs::read_to_string(CONFIG_FILE_PATH)
            .expect("Failed to read config file");
        serde_yaml::from_str(&contents)
            .expect("Failed to parse config file")
    };
}

pub fn get_config() -> &'static Config {
    &CONFIG
}


pub fn get_relay_addr() -> String {
    format!("/ip4/{}/tcp/{}/p2p/{}/{}", CONFIG.server.domain, CONFIG.server.port, CONFIG.id.group_id, CONFIG.server.p2p_relay_addr)
}

pub fn get_nickname() -> String {
    CONFIG.id.nickname.clone()
}

pub fn get_workspace() -> String {
    CONFIG.file.workspace.clone()
}

// Setter to update and save the configuration
pub fn set_config(workspace: String, group_name: String, nickname: String) {
    let new_config = Config {
        server: ServerConfig {
            domain: "".to_string(),
            port: 4001,
            p2p_relay_addr: "".to_string(),
        },
        file: FileConfig {
            workspace,
            refresh_time: 600,
            ignore_list: vec![],
        },
        id: IdConfig {
            group_id: Uuid::new_v5(&Uuid::NAMESPACE_OID, group_name.as_bytes()),
            my_id: Uuid::new_v4(),
            nickname,
        },
    };

    let yaml_content = serde_yaml::to_string(&new_config).expect("Failed to serialize config");

    let mut file = std::fs::File::create(CONFIG_FILE_PATH).expect("Failed to create config file");
    file.write_all(yaml_content.as_bytes())
        .expect("Failed to write to config file");

    println!("Configuration successfully updated!");
}