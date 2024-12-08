extern crate walkdir;

use crate::config_loader::get_config;
use crate::event::ClientEvent;
use kudrive_common::fs::{File, FileMap};
use notify::{Event, RecommendedWatcher, RecursiveMode, Result, Watcher};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use tokio::sync::mpsc::Sender;
use walkdir::WalkDir;

pub struct FileServer {
    responder: Sender<ClientEvent>,
}

fn list_all_files<P: AsRef<Path>>(path: P) -> FileMap {
    let mut files = Vec::new();

    let walker = WalkDir::new(path);

    let iterator = walker.into_iter();

    let filtered_iterator = iterator.filter_map(|e| e.ok());

    for entry in filtered_iterator {
        if entry.file_type().is_file() {
            let file_path = entry.path().display().to_string();
            files.push(file_path);
        }
    }

    let files = files
        .iter()
        .map(|file| File {
            name: file.to_string(),
        })
        .collect();

    return FileMap { files };
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
        let files = list_all_files(get_config().file.workspace.clone());
        let event = ClientEvent::FileMapUpdate { file_map: files };

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
