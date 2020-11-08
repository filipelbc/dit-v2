use anyhow::{Context, Result};
use log::{debug, trace};
use std::fs;
use std::path::PathBuf;
use toml;

use crate::models::{LogEntry, Repository, Task, TaskData};
use crate::utils::directory;
use crate::utils::time::LocalDateTime;

pub struct Repo {
    directory: PathBuf,
}

impl Repository for Repo {
    fn new(directory: PathBuf) -> Self {
        Repo { directory }
    }

    fn resolve_key(&self, key: &str) -> String {
        key.to_string()
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
            .with_context(|| format!("Could not serialize task: {}", task.id))?;
        fs::write(f, s).with_context(|| format!("Could not write to file: {}", task.id))
    }

    fn load(&self, id: &String) -> Result<Task> {
        debug!("Loading task: {}", id);

        let f = self.path(&id);

        trace!("Reading from file: {}", f.display());

        let s = fs::read(&f).with_context(|| format!("Could not read file: {}", f.display()))?;

        let data: TaskData = toml::from_slice(s.as_slice())
            .with_context(|| format!("Could not parse file: {}", f.display()))?;

        Ok(Task::from_data(id.clone(), data))
    }

    fn clock_in(&self, id: &String, now: LocalDateTime) -> Result<()> {
        let mut task = self.load(id)?;
        task.add_log_entry(LogEntry::new(now));
        self.save(&task)
    }
}

impl Repo {
    fn path(&self, id: &String) -> PathBuf {
        self.directory.join(id).with_extension("toml")
    }
}
