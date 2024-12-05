use std::collections::HashMap;
extern crate dirs;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

static mut NICKNAME: Option<String> = None;
static mut WORKSPACE: Option<String> = None;

#[tauri::command]
fn get_nick() -> Option<String> {
    let nickname;
    unsafe {
        nickname = NICKNAME.clone();
    }
    nickname
}

#[tauri::command]
fn set_setting(nickname: String, workspace: String) {
    unsafe {
        NICKNAME = Some(nickname);
        WORKSPACE = Some(workspace);
    }
}

#[tauri::command]
fn init() {
    unsafe {
        NICKNAME = None;
        WORKSPACE = None;
    }
}

#[derive(serde::Serialize)]
struct DirectoryContents {
    files: Vec<String>,
    folders: Vec<String>,
}

#[tauri::command]
fn get_files(path: String) -> DirectoryContents {
    let resolved_path = if path.starts_with("~") {
        let home_dir = dirs::home_dir().expect("Failed to get home directory");
        let home_dir = home_dir.to_str().unwrap().replace("\\", "/");
        path.replacen("~", &home_dir, 1)
    } else {
        path
    };

    println!("Resolved path: {}", resolved_path);

    let mut files = Vec::new();
    let mut folders = Vec::new();

    // 디렉토리 읽기
    let paths = std::fs::read_dir(&resolved_path).expect("Failed to read directory");
    
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
fn get_destinations() -> HashMap<String, Vec<String>> {
    let mut data = HashMap::new();
let file1 = vec![
    "a/".to_string(),
    "a/b/".to_string(),
    "a/c/".to_string(),
    "a/b/c/".to_string(),
    "a/v/c/".to_string(),
    "a/c/d/f/".to_string(),
    "b/".to_string(),
    "b/x/".to_string(),
    "b/y/z/".to_string(),
    "b/x/w/".to_string(),
    "c/d/e/".to_string(),
    "c/d/e/f/".to_string(),
    "c/d/g/h/".to_string(),
    "d/e/f/".to_string(),
    "e/f/g/h/i/".to_string(),
    "f/g/h/".to_string(),
    "f/g/h/j/k/".to_string(),
    "g/h/".to_string(),
    "g/h/i/".to_string(),
    "g/h/j/".to_string(),
    "h/i/".to_string(),
    "h/i/j/".to_string(),
    "i/j/k/l/".to_string(),
    "i/j/l/m/".to_string(),
];
    let file2 = file1.clone();     
    data.insert("1".to_string(), file1);
    data.insert("2".to_string(), file2);
    data
}

#[tauri::command]
fn send_file(from: String, id: String, dest : String) {
    let from = if from.starts_with("~") {
        let home_dir = dirs::home_dir().expect("Failed to get home directory");
        let home_dir = home_dir.to_str().unwrap().replace("\\", "/");
        from.replacen("~", &home_dir, 1)
    } else {
        from
    };
    
    println!("from {} to {} who {}", from , dest, id);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_nick,
            set_setting,
            init,
            get_files,
            get_destinations,
            send_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
