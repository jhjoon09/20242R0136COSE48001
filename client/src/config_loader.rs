use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
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
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileConfig {
    pub workspace: String,
    pub refresh_time: u32,
    pub ignore_list: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IdConfig {
    pub group_id: Uuid,
    pub my_id: Uuid,
    pub nickname: String,
}

const CONFIG_FILE_PATH: &str = "./client/config/client.yaml"; // YAML 파일 경로

lazy_static! {
    static ref CONFIG: Config = {
        let contents = fs::read_to_string(CONFIG_FILE_PATH)
            .expect("Failed to read config file");
        serde_yaml::from_str(&contents)
            .expect("Failed to parse config file")
    };
}

// Getter for the configuration
pub fn get_config() -> &'static Config {
    &CONFIG
}

// Setter to update and save the configuration
pub fn set_config(workspace: String, group_name: String, nickname: String) {
    let new_config = Config {
        server: ServerConfig {
            domain: "localhost".to_string(),
            port: 8080,
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

    let mut file = File::create(CONFIG_FILE_PATH).expect("Failed to create config file");
    file.write_all(yaml_content.as_bytes())
        .expect("Failed to write to config file");

    println!("Configuration successfully updated!");
}
