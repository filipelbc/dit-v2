use log::{trace, debug};
use std::fs;
use std::path::PathBuf;
use toml;

use crate::utils::directory;
use crate::models::{Repository, Task};

pub struct Repo {
    directory: PathBuf,
}

impl Repository for Repo {
    fn new(directory: PathBuf) -> Self {
        Repo { directory }
    }

    fn resolve_key(&self, key: &str) -> String {
        String::from(key)
    }

    fn exists(&self, id: &String) -> bool {
        self.path(&id).exists()
    }

    fn save(&self, task: &Task) -> Result<(), String> {
        debug!("Saving task: {}", task.id);

        let f = self.path(&task.id);

        trace!("Writing to file: {}", f.display());

        let p = f.parent().unwrap();
        directory::ensure_exists(p)?;

        let s = toml::to_string_pretty(&task.data).map_err(|e| e.to_string())?;
        fs::write(f, s).map_err(|e| e.to_string())
    }
}

impl Repo {
    fn path(&self, id: &String) -> PathBuf {
        self.directory.join(id).with_extension("toml")
    }
}
