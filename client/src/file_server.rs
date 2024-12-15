use crate::config_loader::{get_home_dir, get_ignore_list, get_refresh_time, get_workspace};
use crate::event::ClientEvent;
use kudrive_common::fs::{OS, File, FileMap, Folder};
use notify::{Event, RecommendedWatcher, RecursiveMode, Result, Watcher};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use tokio::sync::mpsc::Sender;

pub struct FileServer {
    responder: Sender<ClientEvent>,
}

pub fn resolve_path(path: String) -> String {
    if path.starts_with("~") {
        let home_dir = get_home_dir().to_str().expect("Failed to get home dir");
        path.replacen("~", home_dir, 1)
    } else {
        path
    }
}

fn remove_home(path: String) -> String {
    let workspace = get_workspace();
    let (_, after) = path.split_at(workspace.len());
    format!("{}{}", "home", after)
}

fn get_files(path: String) -> FileMap {
    let path_name = resolve_path(path);

    let os = OS { name: std::env::consts::OS.to_string() };
    let mut files = Vec::new();
    let mut folders = Vec::new();

    let paths = match std::fs::read_dir(&path_name) {
        Ok(entries) => entries,
        Err(_) => return FileMap { os, files, folders },
    };

    for entry in paths {
        if let Ok(entry) = entry {
            let path = entry.path();
            let file_name = match path.file_name() {
                Some(name) => name.to_str().unwrap().to_string(),
                None => continue,
            };

            let file_name = format!("{}/{}", path_name, file_name);

            if path.is_dir() {
                folders.push(Folder {
                    name: format!("{}", file_name),
                });
            } else {
                files.push(File { name: file_name });
            }
        }
    }

    FileMap { os, files, folders }
}

fn get_filemap(path: String) -> FileMap {
    let os =  OS { name: std::env::consts::OS.to_string() };
    let mut all_files = Vec::new();
    let mut all_folders = Vec::new();
    let current = get_files(path.clone());

    all_files.extend(current.files);
    all_folders.extend(current.folders.clone());

    for folder in current.folders {
        let sub_contents = get_filemap(folder.name);
        all_files.extend(sub_contents.files);
        all_folders.extend(sub_contents.folders);
    }

    FileMap {
        os: os,
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

pub fn get_resolved_filemap() -> FileMap {
    let files = get_filemap(get_workspace());

    let os =  OS { name: std::env::consts::OS.to_string() };

    let all_files: Vec<File> = files
        .files
        .into_iter()
        .map(|f| File {
            name: remove_home(f.name),
        })
        .collect();

    let mut all_folders: Vec<Folder> = files
        .folders
        .into_iter()
        .map(|f| Folder {
            name: remove_home(f.name),
        })
        .collect();

    all_folders.push(Folder {
        name: "home/".to_string(),
    });

    FileMap {
        os: os,
        files: all_files,
        folders: all_folders,
    }
}

impl FileServer {
    pub fn new(responder: Sender<ClientEvent>) -> Self {
        Self { responder }
    }

    fn responder(&self) -> Sender<ClientEvent> {
        self.responder.clone()
    }

    async fn send(responder: Sender<ClientEvent>) {
        let files = get_resolved_filemap();

        let event = ClientEvent::FileMapUpdate {
            file_map: files,
        };

        // TODO: logics for file map update
        responder.send(event).await.unwrap();
    }

    pub async fn start(&self) {
        let responder = self.responder();

        tokio::spawn(async move {
            FileServer::send(responder.clone()).await;

            tracing::info!("File server started.");

            let path = get_workspace();
            let exclude_patterns = get_ignore_list()
                .iter()
                .map(|pattern| Regex::new(pattern).unwrap())
                .collect::<Vec<Regex>>();
            let delay_time = get_refresh_time() as u128;

            let (tx, rx) = mpsc::channel::<Result<Event>>();
            let mut watcher: RecommendedWatcher =
                notify::recommended_watcher(tx).expect("watcher error");

            watcher
                .watch(Path::new(&path), RecursiveMode::Recursive)
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
                            notify::EventKind::Access(_any) => {
                                tracing::info!("accessed");
                                continue;
                            }
                            notify::EventKind::Create(_any) => {
                                tracing::info!("created");
                            }

                            notify::EventKind::Modify(_any) => {
                                tracing::info!("modified");
                            }

                            notify::EventKind::Remove(_any) => {
                                tracing::info!("removed");
                            }

                            notify::EventKind::Other => {
                                tracing::info!("other");
                            }

                            notify::EventKind::Any => {
                                tracing::info!("any");
                                continue;
                            }
                        }

                        now = std::time::Instant::now();

                        FileServer::send(responder.clone()).await;
                    }
                    Err(e) => {
                        tracing::error!("watch error: {:?}", e);
                    }
                }
            }
        });
    }

    pub async fn stop(&self) {
        tracing::info!("File server stopped.");
    }
}
