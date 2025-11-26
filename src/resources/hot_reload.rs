use notify::{Config, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};

pub struct HotReloadService {
    rx: Receiver<PathBuf>,
    _watcher: RecommendedWatcher,
}

impl HotReloadService {
    pub fn watch_dir(path: PathBuf) -> NotifyResult<Self> {
        let (tx, rx) = mpsc::channel();
        let mut watcher = RecommendedWatcher::new(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                for p in event.paths { let _ = tx.send(p); }
            }
        }, Config::default())?;
        watcher.watch(&path, RecursiveMode::Recursive)?;
        Ok(Self { rx, _watcher: watcher })
    }
    pub fn poll(&self) -> Option<PathBuf> { self.rx.try_recv().ok() }
}
