use crate::db::Database;
use crate::results::ResultsError;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

pub struct ResultsWatcher {
    shutdown: Mutex<Option<oneshot::Sender<()>>>,
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl ResultsWatcher {
    pub fn new() -> Self {
        Self {
            shutdown: Mutex::new(None),
            handle: Mutex::new(None),
        }
    }

    pub fn is_running(&self) -> bool {
        self.handle
            .lock()
            .map(|h| h.is_some())
            .unwrap_or(false)
    }

    pub fn start(&self, results_dir: PathBuf, db: Arc<Mutex<Database>>) {
        self.stop();

        let (tx, mut rx) = oneshot::channel();
        let handle = tokio::spawn(async move {
            let mut seen = HashSet::new();
            let mut pending: Option<(String, tokio::time::Instant)> = None;

            loop {
                if rx.try_recv().is_ok() {
                    break;
                }

                tokio::time::sleep(Duration::from_secs(2)).await;

                if !results_dir.exists() {
                    continue;
                }

                let Ok(entries) = fs::read_dir(&results_dir) else {
                    continue;
                };

                let mut newest: Option<(String, std::time::SystemTime)> = None;
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) != Some("json") {
                        continue;
                    }
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    if seen.contains(&name) {
                        continue;
                    }
                    let Ok(meta) = entry.metadata() else {
                        continue;
                    };
                    let Ok(modified) = meta.modified() else {
                        continue;
                    };
                    if newest
                        .as_ref()
                        .map(|(_, t)| modified > *t)
                        .unwrap_or(true)
                    {
                        newest = Some((name.clone(), modified));
                    }
                }

                if let Some((name, _)) = newest {
                    let ready = if pending.as_ref().map(|(n, _)| n == &name).unwrap_or(false) {
                        pending
                            .as_ref()
                            .map(|(_, t)| t.elapsed() >= Duration::from_secs(5))
                            .unwrap_or(false)
                    } else {
                        pending = Some((name.clone(), tokio::time::Instant::now()));
                        false
                    };

                    if ready {
                        let file_path = results_dir.join(&name);
                        if let Ok(raw) = fs::read_to_string(&file_path) {
                            if let Ok(database) = db.lock() {
                                if let Ok(result) = database.import_results_file(&name, &raw, "auto")
                                {
                                    if result.success {
                                        seen.insert(name);
                                    }
                                }
                            }
                        }
                        pending = None;
                    }
                }
            }
        });

        if let Ok(mut guard) = self.shutdown.lock() {
            *guard = Some(tx);
        }
        if let Ok(mut guard) = self.handle.lock() {
            *guard = Some(handle);
        }
    }

    pub fn stop(&self) {
        if let Ok(mut guard) = self.shutdown.lock() {
            if let Some(tx) = guard.take() {
                let _ = tx.send(());
            }
        }
        if let Ok(mut guard) = self.handle.lock() {
            if let Some(handle) = guard.take() {
                handle.abort();
            }
        }
    }
}

pub fn results_dir_for_server(server_root: &Path) -> PathBuf {
    server_root.join("results")
}

pub fn import_json(
    db: &mut Database,
    file_name: &str,
    raw: &str,
    source: &str,
) -> Result<crate::models::ImportResult, ResultsError> {
    db.import_results_file(file_name, raw, source)
        .map_err(|e| ResultsError::Message(e.to_string()))
}
