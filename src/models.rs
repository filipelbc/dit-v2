use anyhow::Result;
use chrono::{Duration, Local};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use crate::utils::time::LocalDateTime;

pub struct Task {
    pub id: String,
    pub data: TaskData,
}

#[derive(Serialize, Deserialize)]
pub struct TaskData {
    pub title: String,
    #[serde(default)]
    pub log: Vec<LogEntry>,
}

#[derive(Serialize, Deserialize, Clone, Eq)]
pub struct LogEntry {
    pub start: LocalDateTime,
    pub end: Option<LocalDateTime>,
}

pub struct Status {
    pub id: String,
    pub title: String,
    pub log_entry: LogEntry,
    pub time_spent: Duration,
}

impl Task {
    pub fn new(id: String) -> Task {
        Task {
            id,
            data: TaskData {
                title: String::new(),
                log: Vec::new(),
            },
        }
    }

    pub fn from_data(id: String, data: TaskData) -> Task {
        Task { id, data }
    }

    pub fn validate_key(key: &str) -> Result<(), String> {
        match TASK_KEY.is_match(key) {
            true => Ok(()),
            false => Err(key.to_string()),
        }
    }
}

impl LogEntry {
    pub fn new(start: LocalDateTime) -> LogEntry {
        LogEntry { start, end: None }
    }

    pub fn is_open(&self) -> bool {
        self.end.is_none()
    }

    pub fn is_closed(&self) -> bool {
        self.end.is_some()
    }
}

impl Status {
    pub fn start(&self) -> LocalDateTime {
        self.log_entry.start
    }

    pub fn end(&self) -> LocalDateTime {
        self.log_entry.end.unwrap_or_else(|| Local::now())
    }

    pub fn duration(&self) -> Duration {
        self.end() - self.start()
    }
}

pub trait Repository {
    fn resolve_key(&self, key: &str) -> String;
    fn exists(&self, id: &String) -> bool;
    fn save(&self, task: &Task) -> Result<()>;
    fn load(&self, id: &String) -> Result<Task>;
    fn clock_in(&self, id: &String, now: LocalDateTime) -> Result<()>;
    fn clock_out(&self, id: &String, now: LocalDateTime) -> Result<()>;
    fn un_clock_in(&self, id: &String) -> Result<()>;
    fn un_clock_out(&self, id: &String) -> Result<()>;
    fn is_clocked_in(&self) -> Option<String>;
    fn current_task(&self) -> Option<(String, LogEntry)>;
    fn get_status(&self, limit: usize) -> Vec<Status>;
}

impl Ord for LogEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start)
    }
}

impl PartialOrd for LogEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for LogEntry {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start
    }
}

lazy_static! {
    static ref TASK_KEY: Regex = Regex::new(r"^(/?[A-Za-z][0-9A-Za-z_-]*)+$").unwrap();
}

#[cfg(test)]
mod tests {

    use super::Task;

    macro_rules! assert_valid_key {
        ($expr:expr) => {{
            if let Err(_) = Task::validate_key($expr) {
                panic!(
                    "assertion failed: {} was expected to be a valid key",
                    stringify!($expr),
                );
            }
        }};
    }

    macro_rules! assert_invalid_key {
        ($expr:expr) => {{
            if let Ok(_) = Task::validate_key($expr) {
                panic!(
                    "assertion failed: {} was not expected to be valid",
                    stringify!($expr),
                );
            }
        }};
    }

    #[test]
    fn test_validate_task_key() {
        assert_valid_key!("f");
        assert_valid_key!("/f");
        assert_valid_key!("foo");
        assert_valid_key!("/foo");
        assert_valid_key!("/foo/bar");
        assert_valid_key!("foo/bar");
        assert_valid_key!("f_O/b-R");
        assert_valid_key!("f_O");
        assert_valid_key!("f-O");
        assert_valid_key!("/f_O");
        assert_valid_key!("/f-O");

        assert_invalid_key!("_oo");
        assert_invalid_key!("-oo");
        assert_invalid_key!("foo/-ar");
        assert_invalid_key!("foo/_ar");
        assert_invalid_key!("foo/bar/");
        assert_invalid_key!("foo.bar");
        assert_invalid_key!(".foo/bar");
    }
}
