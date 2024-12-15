use serde::{Deserialize, Serialize};
use serde_yaml::{from_str, to_string};
use std::path::PathBuf;
use std::{fs, io::Write, path::Path};
use tokio::sync::OnceCell;
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
    pub server_port: u16,
    pub p2p_port: u16,
    pub hash: String,
    pub p2p_relay_addr: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileConfig {
    pub workspace: String,
    pub refresh_time: u128,
    pub ignore_list: Vec<String>, // ignore_list를 Vec<String>으로 매핑
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IdConfig {
    pub group_id: Uuid,
    pub my_id: Uuid,
    pub nickname: String,
}

static APP_DATA_DIR: OnceCell<PathBuf> = OnceCell::const_new();
static HOME_DIR: OnceCell<PathBuf> = OnceCell::const_new();
static CONFIG: OnceCell<Config> = OnceCell::const_new();

pub async fn set_data_dir(save_dir: PathBuf) -> &'static PathBuf {
    APP_DATA_DIR
        .get_or_init(|| async {
            let path = save_dir.clone();
            path.join("config.yaml")
        })
        .await
}

pub async fn set_home_dir(home_dir: PathBuf) -> &'static PathBuf {
    HOME_DIR.get_or_init(|| async { home_dir.clone() }).await
}

pub async fn init_config() -> &'static Config {
    CONFIG
        .get_or_init(|| async {
            let path = get_data_dir();
            let contents = fs::read_to_string(&path).expect("Failed to read config file");
            from_str::<Config>(&contents).expect("Failed to parse config file")
        })
        .await
}

pub fn get_data_dir() -> &'static PathBuf {
    APP_DATA_DIR.get().expect("Data directory not found")
}

pub fn get_home_dir() -> &'static PathBuf {
    HOME_DIR.get().expect("Home directory not found")
}

fn get_config() -> &'static Config {
    CONFIG.get().expect("Config not found")
}

pub async fn set_config_path(save_dir: PathBuf, home_dir: PathBuf) {
    set_data_dir(save_dir).await;
    set_home_dir(home_dir).await;
}

pub async fn is_first_run() -> bool {
    let path = get_data_dir();
    !(path.exists() && path.is_file())
}

pub fn get_relay_addr() -> String {
    let config = get_config();
    config.server.p2p_relay_addr.clone()
}

pub fn get_nickname() -> String {
    let config = get_config();
    config.id.nickname.clone()
}

pub fn get_group_id() -> Uuid {
    let config = get_config();
    config.id.group_id.clone()
}

pub fn get_server_address() -> String {
    let config = get_config();
    format!("{}:{}", config.server.domain, config.server.server_port)
}

pub fn get_ignore_list() -> Vec<String> {
    let config = get_config();
    config.file.ignore_list.clone()
}

pub fn get_refresh_time() -> u128 {
    let config = get_config();
    config.file.refresh_time
}

pub fn get_uuid() -> Uuid {
    let config = get_config();
    config.id.my_id.clone()
}

pub fn get_workspace() -> String {
    let config = get_config();
    config.file.workspace.clone()
}

// Setter to update and save the configuration
pub async fn set_config(
    workspace: String,
    group_name: String,
    nickname: String,
    domain: Option<String>,
    hash: Option<String>,
    server_port: Option<u16>,
    p2p_port: Option<u16>,
) -> Result<(), String> {
    let domain = domain.unwrap_or("127.0.0.1".to_string());
    let server_port = server_port.unwrap_or(7878);
    let p2p_port = p2p_port.unwrap_or(4001);
    let hash = hash.unwrap_or("12D3KooWA768LzHMatxkjD1f9DrYW375GZJr6MHPCNEdDtHeTNRt".to_string());

    let new_config = Config {
        server: ServerConfig {
            domain: domain.clone(),
            server_port: server_port,
            p2p_port: p2p_port,
            hash: hash.clone(),
            p2p_relay_addr: format!(
                "/ip4/{}/tcp/{}/p2p/{}",
                domain.clone(),
                p2p_port,
                hash.clone()
            ),
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

    let yaml_content = to_string(&new_config).expect("Failed to serialize config");

    let file_pos = get_data_dir();

    let path = Path::new(&file_pos);

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).expect("Failed to create config directory");
        }
    }

    let mut file = std::fs::File::create(path).expect("Failed to create config file");
    file.write_all(yaml_content.as_bytes())
        .expect("Failed to write to config file");

    tracing::info!("Configuration successfully updated!");

    Ok(())
}

pub fn get_current_config() -> Result<&'static Config, String> {
    CONFIG
        .get()
        .ok_or_else(|| "설정이 초기화되지 않았습니다".to_string())
}
