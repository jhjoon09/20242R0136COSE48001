extern crate walkdir;

use crate::config_loader::get_config;
use kudrive_common::event::client::ClientEvent;
use kudrive_common::fs::{File, Folder, FileMap};
use notify::{Event, RecommendedWatcher, RecursiveMode, Result, Watcher};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use tokio::sync::mpsc::Sender;
use dirs;

pub struct FileServer {
    responder: Sender<ClientEvent>,
}

fn get_files(path : String) -> FileMap {
    let resolved_path = if path.starts_with("~") {
        let home_dir = dirs::home_dir().expect("Failed to get home directory");
        let home_dir = home_dir.to_str().unwrap().replace("\\", "/");
        path.replacen("~", &home_dir, 1)
    } else {
        path
    };

    let mut files = Vec::new();
    let mut folders = Vec::new();

    let paths = match std::fs::read_dir(&resolved_path) {
        Ok(entries) => entries,
        Err(_) => return FileMap { files, folders },
    };

    for entry in paths {
        if let Ok(entry) = entry {
            let path = entry.path();
            let file_name = match path.file_name() {
                Some(name) => name.to_str().unwrap().to_string(),
                None => continue,
            };

            let file_name = format!("{}/{}", resolved_path, file_name);

            if path.is_dir() {
                folders.push(Folder { name : file_name });
            } else {
                files.push(File { name : file_name });
            }
        }
    }

    FileMap { files, folders }
}

fn get_filemap(path: String) -> FileMap {
    let mut all_files = Vec::new();
    let mut all_folders = Vec::new();

    // 현재 경로의 파일 및 폴더 가져오기
    let current = get_files(path.clone());

    // 현재 폴더와 파일 추가
    all_files.extend(current.files);
    all_folders.extend(current.folders.clone());

    // 하위 폴더를 재귀적으로 탐색
    for folder in current.folders {
        let sub_contents = get_filemap(folder.name);
        all_files.extend(sub_contents.files);
        all_folders.extend(sub_contents.folders);
    }

    FileMap {
        files: all_files,
        folders: all_folders,
    }
}


fn check_exclude(path: &PathBuf, patterns: &[Regex]) -> bool {
    path.components().any(|component| {
        let component_str = component.as_os_str().to_string_lossy();
        patterns
            .iter()
            .any(|pattern| pattern.is_match(&component_str))
    })
}

impl FileServer {
    pub fn new(responder: Sender<ClientEvent>) -> Self {
        Self { responder }
    }

    fn responder(&self) -> Sender<ClientEvent> {
        self.responder.clone()
    }

    async fn send(responder: Sender<ClientEvent>) {
        let file_map = get_filemap(get_config().file.workspace.clone());
        let event = ClientEvent::FileMapUpdate { file_map: file_map.clone() };

        // TODO: logics for file map update
        responder.send(event).await.unwrap();
    }

    pub async fn start(&self) {
        let responder = self.responder();

        FileServer::send(responder.clone()).await;

        tokio::spawn(async move {
            println!("File server started.");
            let config = get_config();
            let path = &config.file.workspace;
            let exclude_patterns = config
                .file
                .ignore_list
                .iter()
                .map(|pattern| Regex::new(pattern).unwrap())
                .collect::<Vec<Regex>>();
            let delay_time = config.file.refresh_time as u128;
            let (tx, rx) = mpsc::channel::<Result<Event>>();
            let mut watcher: RecommendedWatcher =
                notify::recommended_watcher(tx).expect("watcher error");

            watcher
                .watch(Path::new(path), RecursiveMode::Recursive)
                .expect("watch error");

            let mut now = std::time::Instant::now();

            for res in rx {
                match res {
                    Ok(event) => {
                        let elapsed = now.elapsed();
                        if elapsed.as_millis() < delay_time {
                            continue;
                        }

                        // Skip if any path in event matches any exclude pattern
                        if event
                            .paths
                            .iter()
                            .any(|path| check_exclude(&path, &exclude_patterns))
                        {
                            continue;
                        }

                        match event.kind {
                            notify::EventKind::Create(_any) => {
                                println!("created");
                            }

                            notify::EventKind::Remove(_any) => {
                                println!("removed");
                            }

                            _ => {
                                continue;
                            }
                        }

                        now = std::time::Instant::now();

                        FileServer::send(responder.clone()).await;
                    }
                    Err(e) => {
                        eprintln!("watch error: {:?}", e);
                    }
                }
            }
        });
    }

    pub async fn stop(&self) {
        println!("File server stopped.");
    }
}
