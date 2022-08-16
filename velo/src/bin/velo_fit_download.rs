use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::FileType;
use std::path::{Path, PathBuf};
use std::thread::{self, Thread};
use std::time::Duration;

use log::{debug, error, info};

use notify::event::CreateKind;
use velo::sqlite::Db;
use velo::wahoo::{self, WahooWebhook};
use velo::AppConfig;

pub fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let config = AppConfig::from_env().expect("Failed to load AppConfig from env");
    if config.sqlite_directory.exists() == false {
        std::fs::create_dir_all(&config.sqlite_directory)?;
    }

    // Create a channel to receive the events.
    let (watcher, file_events) = {
        use notify::*;
        use std::time::Duration;

        let (tx, rx) = crossbeam_channel::unbounded();

        // Create a watcher object, delivering debounced events.
        // The notification back-end is selected based on the platform.
        let mut watcher = notify::recommended_watcher(tx)?;
        watcher.configure(Config::PreciseEvents(true))?;

        info!("Watching {} for changes", config.sqlite_directory.display());
        watcher.watch(&config.sqlite_directory, RecursiveMode::NonRecursive)?;

        (watcher, rx)
    };

    let new_db_files = file_events.iter().filter_map(|e| {
        let e = e.expect("File Watcher failed");
        match e.kind {
            notify::EventKind::Create(CreateKind::File) => Some(e.paths[0].clone()),
            _ => None,
        }
    });

    let mut dbs = HashMap::<PathBuf, Db>::new();

    for file in new_db_files {
        debug!("Discovered {}", file.display());

        if file.extension() == Some(OsStr::new("sqlite")) && dbs.contains_key(&file) == false {
            info!("Spawning database-watcher for {}", file.display());

            let mut db = Db::new(&file)?;
            let file2 = file.clone();
            db.raw().commit_hook(Some(move || {
                info!("Got commit in {}", file2.display());
                false
            }));

            dbs.insert(file, db);

            // let t = thread::spawn(move || Ok(()));

            info!("Now watching {} sqlite databases", dbs.len());
        }
    }

    Ok(())
}

struct DatabaseWatcher {
    watcher: notify::RecommendedWatcher,
    databases: HashMap<PathBuf, Db>,
}

impl DatabaseWatcher {
    pub fn new(directory: impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        // Create a channel to receive the events.
        let (watcher, file_events) = {
            use notify::*;
            use std::time::Duration;

            let (tx, rx) = crossbeam_channel::unbounded();

            // Create a watcher object, delivering debounced events.
            // The notification back-end is selected based on the platform.
            let mut watcher = notify::recommended_watcher(tx)?;
            watcher.configure(Config::PreciseEvents(true))?;

            info!("Watching {} for changes", directory.as_ref().display());
            watcher.watch(directory.as_ref(), RecursiveMode::NonRecursive)?;

            (watcher, rx)
        };

        // let new_db_files = file_events.iter().filter_map(|e| {
        //     let e = e.expect("File Watcher failed");
        //     match e.kind {
        //         notify::EventKind::Create(CreateKind::File) => Some(e.paths[0].clone()),
        //         _ => None,
        //     }
        // });

        let mut handler = DatabaseWatcher {
            watcher: watcher,
            databases: HashMap::new(),
        };

        for entry in std::fs::read_dir(directory.as_ref())? {
            handler.handle_path_event(entry?.path())?
        }

        Ok(handler)
    }

    fn is_sqlite_db(path: impl AsRef<Path>) -> bool {
        path.as_ref().extension() == Some(OsStr::new("sqlite"))
    }

    fn handle_path_event(&mut self, path: impl AsRef<Path>) -> Result<(), anyhow::Error> {
        if Self::is_sqlite_db(&path) && !self.databases.contains_key(path.as_ref()) {
            self.databases
                .insert(path.as_ref().to_path_buf(), Db::new(&path)?);
        }

        Ok(())
    }
}
