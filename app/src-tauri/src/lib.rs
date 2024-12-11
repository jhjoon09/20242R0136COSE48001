use dirs;
use std::collections::HashMap;
use std::path::Path;
use tokio::sync::Mutex;
use uuid::Uuid;

use std::sync::{Arc, LazyLock};

use kudrive_client::config_loader::{get_uuid, set_config};
use kudrive_client::init as client_init;
use kudrive_client::{clients, file_receive, file_send};
use tracing_subscriber::EnvFilter;

const CONFIG_FILE_PATH: &str = "./client.yaml";

static GLOBAL_STATE: LazyLock<Arc<Mutex<bool>>> = LazyLock::new(|| Arc::new(Mutex::new(true)));

fn resolve_path(path: String) -> String {
    if path.starts_with("~") {
        let home_dir = dirs::home_dir().expect("Failed to get home directory");
        let home_dir = home_dir.to_str().unwrap().replace("\\", "/");
        path.replacen("~", &home_dir, 1)
    } else {
        path
    }
}

#[tauri::command]
fn is_first_run() -> bool {
    let path = Path::new(CONFIG_FILE_PATH);
    !(path.exists() && path.is_file())
}

#[tauri::command]
fn init_config(workspace: String, group: String, nickname: String) {
    let workspace = resolve_path(workspace);
    set_config(workspace, group, nickname);
}

#[tauri::command]
fn get_nickname() -> String {
    let config = kudrive_client::config_loader::get_config();
    config.id.nickname.clone()
}

#[tauri::command]
async fn init_client() {
    let mut is_first = GLOBAL_STATE.lock().await;
    if *is_first {
        client_init().await;
        *is_first = false;
        drop(is_first);
        return;
    }

    return;
}

#[derive(serde::Serialize)]
struct DirectoryContents {
    files: Vec<String>,
    folders: Vec<String>,
}

#[tauri::command]
fn get_files(path: String) -> DirectoryContents {
    let path = resolve_path(path);

    let mut files = Vec::new();
    let mut folders = Vec::new();

    // 디렉토리 읽기
    let paths = std::fs::read_dir(&path).expect("Failed to read directory");

    for entry in paths {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();

        if path.is_dir() {
            if std::fs::read_dir(&path).is_ok() {
                folders.push(file_name);
            }
        } else {
            files.push(file_name);
        }
    }

    DirectoryContents { files, folders }
}

#[tauri::command]
async fn get_filemap() -> (HashMap<String, Vec<String>>, Vec<(String, String)>) {
    let client_data = clients().await;

    let mut map = HashMap::new();
    let mut id_map = vec![];
    let my_id = get_uuid();
    match client_data {
        Ok(clients) => {
            for client in clients {
                if client.id == my_id {
                    continue;
                }

                let mut file_vec = vec![];
                for file in client.files.files {
                    file_vec.push(file.name);
                }

                map.insert(client.id.to_string().clone(), file_vec);
                id_map.push((client.nickname.clone(), client.id.clone().to_string()));
            }
            (map, id_map)
        }
        Err(e) => {
            eprintln!("Failed to get clients: {:?}", e);
            (HashMap::new(), vec![])
        }
    }
}

#[tauri::command]
async fn get_foldermap() -> (HashMap<String, Vec<String>>, Vec<(String, String)>) {
    let client_data = clients().await;

    let mut map = HashMap::new();
    let mut id_map = vec![];
    let my_id = get_uuid();

    match client_data {
        Ok(clients) => {
            for client in clients {
                if client.id == my_id {
                    continue;
                }

                let mut folder_vec = vec![];
                for folder in client.files.folders {
                    folder_vec.push(folder.name);
                }

                let key = client.id.clone().to_string();
                map.insert(key, folder_vec);
                id_map.push((client.nickname.clone(), client.id.clone().to_string()));
            }

            println!("{:?}", map);
            (map, id_map)
        }
        Err(e) => {
            eprintln!("Failed to get clients: {:?}", e);
            (HashMap::new(), vec![])
        }
    }
}

#[tauri::command]
async fn send_file(id: Uuid, source: String, target: String) -> Result<(), String> {
    let source = resolve_path(source);

    println!("from {} to {} who {}", source, target, id);

    file_send(id, source, target).await
}

#[tauri::command]
async fn recive_file(id: Uuid, source: String, target: String) -> Result<(), String> {
    let target = resolve_path(target);
    println!("from {} to {} who {}", source, target, id);
    file_receive(id, source, target).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    tauri::Builder::default()
        .setup(|app| {
            tokio::task::spawn(async move {
                loop {
                    // Client core init here...
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    tracing::info!("Hello from the background");
                }
            });
            Ok(())
        })
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            is_first_run,
            init_config,
            get_nickname,
            init_client,
            get_files,
            get_filemap,
            get_foldermap,
            send_file,
            recive_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
