use anyhow::{bail, Context, Result};
use chrono::Duration;
use log::{debug, trace};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use toml;
use walkdir::WalkDir;

use crate::models::{ListItem, LogEntry, Repository, StatusItem, Task, TaskData};
use crate::utils::directory;
use crate::utils::time::Timestamp;

pub struct Repo {
    directory: PathBuf,
    index: RefCell<Index>,
}

type Index = HashMap<String, IndexEntry>;

#[derive(Serialize, Deserialize, Clone)]
struct IndexEntry {
    title: String,
    #[serde(flatten)]
    log_entry: LogEntry,
    #[serde(with = "crate::utils::time::duration")]
    total_effort: Duration,
}

impl IndexEntry {
    fn new(task: &Task, entry: &LogEntry) -> Self {
        IndexEntry {
            title: task.data.title.clone(),
            log_entry: entry.clone(),
            total_effort: task.total_effort(),
        }
    }

    fn to_status(&self, id: &String) -> StatusItem {
        StatusItem {
            id: id.clone(),
            title: self.title.clone(),
            log_entry: self.log_entry.clone(),
            total_effort: self.total_effort.clone(),
        }
    }
}

impl Repository for Repo {
    fn resolve_key(&self, key: &str) -> String {
        key.to_string()
    }

    fn exists(&self, id: &String) -> bool {
        self.path(&id).exists()
    }

    fn save(&self, task: &Task) -> Result<()> {
        debug!("Saving task: {}", task.id);

        write(&self.path(&task.id), &task.data)
            .with_context(|| format!("Could not save task: {}", task.id))?;
        self.update_index(&task);
        self.save_index()
    }

    fn load(&self, id: &String) -> Result<Task> {
        debug!("Loading task: {}", id);

        let mut data: TaskData =
            read(&self.path(&id)).with_context(|| format!("Could not load task: {}", id))?;

        data.log.sort();

        Ok(Task::from_data(id.clone(), data))
    }

    fn clock_in(&self, id: &String, now: Timestamp) -> Result<()> {
        let mut task = self.load(id)?;
        task.data.log.push(LogEntry::new(now));
        self.save(&task)
    }

    fn clock_out(&self, id: &String, now: Timestamp) -> Result<()> {
        let mut task = self.load(id)?;
        match task.data.log.last_mut() {
            Some(entry) => match entry.end {
                Some(_) => bail!("Log entry already closed"),
                None => entry.end = Some(now),
            },
            None => bail!("No log entry found to close"),
        };
        self.save(&task)
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
        self.save(&task)
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
        self.save(&task)
    }

    fn is_clocked_in(&self) -> Option<String> {
        self.index
            .borrow()
            .iter()
            .find(|(_, v)| v.log_entry.is_open())
            .map(|(k, _)| k.clone())
    }

    fn previous_task(&self, i: usize) -> Option<(String, LogEntry)> {
        self.sorted_index()
            .get(i)
            .map(|(k, v)| (k.clone(), v.log_entry.clone()))
    }

    fn get_status(&self, limit: usize) -> Vec<StatusItem> {
        let mut status: Vec<StatusItem> = self
            .index
            .borrow()
            .iter()
            .map(|(k, v)| v.to_status(k))
            .collect();

        status.sort_unstable_by(|x, y| y.log_entry.cmp(&x.log_entry));
        if limit > 0 {
            status.truncate(limit);
        }
        status
    }

    fn get_listing(
        &self,
        after: Option<Timestamp>,
        before: Option<Timestamp>,
    ) -> Result<Vec<ListItem>> {
        let is_after = |x| after.map(|a| x >= a).unwrap_or(true);
        let is_before = |x| before.map(|b| x <= b).unwrap_or(true);

        let tasks = self
            .sorted_index()
            .iter()
            .filter(|(_, v)| is_after(v.log_entry.start))
            .map(|(id, _)| self.load(id))
            .collect::<Result<Vec<_>, _>>()?;

        let mut items = tasks
            .iter()
            .map(|t| {
                t.data
                    .log
                    .iter()
                    .filter(|e| is_after(e.start) && is_before(e.start))
                    .map(move |e| ListItem::new(t, e))
            })
            .flatten()
            .collect::<Vec<_>>();

        items.sort_unstable_by(|x, y| y.log_entry.cmp(&x.log_entry));

        Ok(items)
    }

    fn rebuild_index(&self) -> Result<()> {
        self.index.borrow_mut().clear();

        for id in self.list_ids()? {
            let t = self.load(&id)?;
            self.update_index(&t);
        }
        self.save_index()
    }
}

impl Repo {
    pub fn new(directory: PathBuf) -> Result<Self> {
        let index = Repo::load_index(&directory)?;
        Repo::check_index(&index)?;
        Ok(Repo {
            directory,
            index: RefCell::new(index),
        })
    }

    fn path(&self, id: &str) -> PathBuf {
        self.directory.join(id).with_extension("toml")
    }

    fn id_from_full_path(&self, path: &PathBuf) -> Result<String> {
        let id = path
            .strip_prefix(&self.directory)
            .with_context(|| {
                format!(
                    "Given path is not a child of dit directory: {}",
                    path.display()
                )
            })?
            .display()
            .to_string()
            .strip_suffix(".toml")
            .with_context(|| {
                format!(
                    "Given path does not have toml extension: {}",
                    path.display()
                )
            })?
            .to_string();
        Ok(id)
    }

    fn list_ids(&self) -> Result<Vec<String>> {
        let mut ids = Vec::new();

        for entry in WalkDir::new(&self.directory) {
            let p = entry
                .with_context(|| {
                    format!(
                        "Could not complete traversal of: {}",
                        &self.directory.display()
                    )
                })?
                .into_path();

            if !p.extension().map(|x| x.eq("toml")).unwrap_or(false) {
                continue;
            }

            if !p.is_file() {
                continue;
            }

            ids.push(self.id_from_full_path(&p)?);
        }
        Ok(ids)
    }

    fn sorted_index(&self) -> Vec<(String, IndexEntry)> {
        let b = self.index.borrow();

        let mut d: Vec<_> = b.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

        d.sort_unstable_by(|x, y| y.1.log_entry.cmp(&x.1.log_entry));

        d
    }

    fn load_index(directory: &Path) -> Result<Index> {
        trace!("Loading index");

        let s = directory.join(".index").with_extension("toml");
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
        trace!("Saving index");

        write(&self.path(".index"), &self.index).context("Could not save index")
    }

    fn update_index(&self, task: &Task) {
        if let Some(entry) = task.data.log.last() {
            self.index
                .borrow_mut()
                .insert(task.id.clone(), IndexEntry::new(task, entry));
        } else {
            self.index.borrow_mut().remove(&task.id);
        }
    }
}

impl Task {
    fn total_effort(&self) -> Duration {
        self.data
            .log
            .iter()
            .fold(Duration::seconds(0), |a, x| a + x.effort())
    }
}

fn read<T: DeserializeOwned>(f: &Path) -> Result<T> {
    trace!("Reading from file: {}", f.display());

    let s =
        fs::read_to_string(&f).with_context(|| format!("Could not read file: {}", f.display()))?;

    toml::from_str(s.as_str()).with_context(|| format!("Could not parse file: {}", f.display()))
}

fn write<T: Serialize>(f: &Path, d: &T) -> Result<()> {
    directory::ensure_exists(f.parent().unwrap())?;

    trace!("Writing to file: {}", f.display());

    let s = toml::to_string_pretty(&d).context("Could not serialize object")?;
    fs::write(f, s).with_context(|| format!("Could not write to file: {}", f.display()))
}
