use anyhow::{bail, Context, Result};
use log::{debug, trace};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
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

#[derive(Serialize, Deserialize)]
struct IndexEntry {
    title: String,
    #[serde(flatten)]
    log_entry: LogEntry,
}

impl IndexEntry {
    pub fn new(task: &Task, entry: &LogEntry) -> Self {
        IndexEntry {
            title: task.data.title.clone(),
            log_entry: entry.clone(),
        }
    }
}

type Index = HashMap<String, IndexEntry>;

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
        let mut data: TaskData =
            read(&f).with_context(|| format!("Could not load task: {}", id))?;

        data.log.sort();

        Ok(Task::from_data(id.clone(), data))
    }

    fn clock_in(&self, id: &String, now: LocalDateTime) -> Result<()> {
        let mut task = self.load(id)?;
        task.data.log.push(LogEntry::new(now));
        self.save(&task)?;
        self.update_index(&task)
    }

    fn clock_out(&self, id: &String, now: LocalDateTime) -> Result<()> {
        let mut task = self.load(id)?;
        match task.data.log.last_mut() {
            Some(entry) => match entry.end {
                Some(_) => bail!("Log entry already closed"),
                None => entry.end = Some(now),
            },
            None => bail!("No log entry found to close"),
        };
        self.save(&task)?;
        self.update_index(&task)
    }

    fn un_clock_in(&self, id: &String) -> Result<()> {
        let mut task = self.load(id)?;
        match task.data.log.last() {
            Some(entry) => match entry.end {
                Some(_) => bail!("Log entry already closed"),
                None => task.data.log.pop(),
            },
            None => bail!("No log entry found to close"),
        };
        self.save(&task)?;
        self.update_index(&task)
    }

    fn un_clock_out(&self, id: &String) -> Result<()> {
        let mut task = self.load(id)?;
        match task.data.log.last_mut() {
            Some(entry) => match entry.end {
                Some(_) => entry.end = None,
                None => bail!("Log entry already open"),
            },
            None => bail!("No log entry found to close"),
        };
        self.save(&task)?;
        self.update_index(&task)
    }

    fn is_clocked_in(&self) -> Option<String> {
        self.index
            .borrow()
            .iter()
            .find(|&(_, v)| v.log_entry.is_open())
            .map(|(k, _)| k.clone())
    }

    fn current_task(&self) -> Option<(String, LogEntry)> {
        self.index
            .borrow()
            .iter()
            .max_by(|&x, &y| x.1.log_entry.cmp(&y.1.log_entry))
            .map(|(k, v)| (k.clone(), v.log_entry.clone()))
    }
}

impl Repo {
    pub fn new(directory: PathBuf) -> Result<Self> {
        let index = Repo::load_index(directory.as_path())?;
        Repo::check_index(&index)?;
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

    fn check_index(index: &Index) -> Result<()> {
        let c = index
            .values()
            .fold(0, |a, i| if i.log_entry.is_open() { a + 1 } else { a });

        if c > 1 {
            bail!("Index contains more than one active task; rebuild index?");
        }
        Ok(())
    }

    fn save_index(&self) -> Result<()> {
        let s = self.directory.join(".index");
        write(&s, &self.index).context("Could not save index")
    }

    fn update_index(&self, task: &Task) -> Result<()> {
        if let Some(entry) = task.data.log.last() {
            self.index
                .borrow_mut()
                .insert(task.id.clone(), IndexEntry::new(task, entry));
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
