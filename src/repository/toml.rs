use anyhow::{bail, Context, Result};
use log::{debug, trace};
use serde::{de::DeserializeOwned, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use toml;

use crate::models::{LogEntry, Repository, Task, TaskData};
use crate::utils::directory;
use crate::utils::time::LocalDateTime;

pub struct Repo {
    directory: PathBuf,
    index: RefCell<Index>,
}

type Index = HashMap<String, LogEntry>;

impl Repository for Repo {
    fn resolve_key(&self, key: &str) -> String {
        key.to_string()
    }

    fn exists(&self, id: &String) -> bool {
        self.path(&id).exists()
    }

    fn save(&self, task: &Task) -> Result<()> {
        debug!("Saving task: {}", task.id);

        let f = self.path(&task.id);
        write(&f, &task.data).with_context(|| format!("Could not save task: {}", task.id))
    }

    fn load(&self, id: &String) -> Result<Task> {
        debug!("Loading task: {}", id);

        let f = self.path(&id);
        let data: TaskData = read(&f).with_context(|| format!("Could not load task: {}", id))?;

        Ok(Task::from_data(id.clone(), data))
    }

    fn clock_in(&self, id: &String, now: LocalDateTime) -> Result<()> {
        let mut task = self.load(id)?;
        task.data.log.push(LogEntry::new(now));
        self.save(&task)?;
        self.update_index(&task)
    }

    fn is_clocked_in(&self) -> Option<String> {
        self.index
            .borrow()
            .iter()
            .find(|&(_, v)| v.end.is_none())
            .map(|(k, _)| k.clone())
    }

    fn clock_out(&self, id: &String, now: LocalDateTime) -> Result<()> {
        let mut task = self.load(id)?;
        match task.data.log.last_mut() {
            Some(entry) => match entry.end {
                Some(_) => bail!("Log entry already closed"),
                None => entry.end = Some(now),
            },
            None => bail!("No log entry found to close"),
        }
        self.save(&task)?;
        self.update_index(&task)
    }
}

impl Repo {
    pub fn new(directory: PathBuf) -> Result<Self> {
        let index = Repo::load_index(directory.as_path())?;
        Ok(Repo {
            directory,
            index: RefCell::new(index),
        })
    }

    fn path(&self, id: &String) -> PathBuf {
        self.directory.join(id).with_extension("toml")
    }

    fn load_index(path: &Path) -> Result<Index> {
        let s = path.join(".index");
        if s.is_file() {
            read(&s)
        } else {
            debug!("Index not found; using new, empty one");
            Ok(Index::new())
        }
    }

    fn save_index(&self) -> Result<()> {
        let s = self.directory.join(".index");
        write(&s, &self.index).context("Could not save index")
    }

    fn update_index(&self, task: &Task) -> Result<()> {
        if let Some(entry) = task.data.log.last() {
            self.index
                .borrow_mut()
                .insert(task.id.clone(), entry.clone());
        } else {
            self.index.borrow_mut().remove(&task.id);
        }
        self.save_index()
    }
}

fn read<T: DeserializeOwned>(f: &PathBuf) -> Result<T> {
    trace!("Reading from file: {}", f.display());

    let s =
        fs::read_to_string(&f).with_context(|| format!("Could not read file: {}", f.display()))?;

    toml::from_str(s.as_str()).with_context(|| format!("Could not parse file: {}", f.display()))
}

fn write<T: Serialize>(f: &PathBuf, d: &T) -> Result<()> {
    trace!("Writing to file: {}", f.display());

    let p = f.parent().unwrap();
    directory::ensure_exists(p)?;

    let s = toml::to_string_pretty(&d).context("Could not serialize object")?;
    fs::write(f, s).with_context(|| format!("Could not write to file: {}", f.display()))
}
