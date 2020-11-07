use anyhow::{Context, Result};
use log::{debug, trace};
use std::fs;
use std::path::PathBuf;
use toml;

use crate::models::{Repository, Task};
use crate::utils::directory;

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

    fn save(&self, task: &Task) -> Result<()> {
        debug!("Saving task: {}", task.id);

        let f = self.path(&task.id);

        trace!("Writing to file: {}", f.display());

        let p = f.parent().unwrap();
        directory::ensure_exists(p)?;

        let s = toml::to_string_pretty(&task.data)
            .with_context(|| format!("Failed to save task: {}", task.id))?;
        fs::write(f, s).with_context(|| format!("Failed to save task: {}", task.id))
    }
}

impl Repo {
    fn path(&self, id: &String) -> PathBuf {
        self.directory.join(id).with_extension("toml")
    }
}
