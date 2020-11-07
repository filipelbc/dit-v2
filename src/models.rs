use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub struct Task {
    pub id: String,
    pub data: TaskData,
}

#[derive(Serialize, Deserialize)]
pub struct TaskData {
    pub title: String,
}

impl Task {
    pub fn new(id: String) -> Task {
        Task {
            id,
            data: TaskData {
                title: String::from("Fixme"),
            },
        }
    }

    pub fn validate_key(key: &str) -> Result<(), String> {
        match TASK_KEY.is_match(key) {
            true => Ok(()),
            false => Err(String::from(key)),
        }
    }
}

pub trait Repository {
    fn new(directory: PathBuf) -> Self;
    fn resolve_key(&self, key: &str) -> String;
    fn exists(&self, id: &String) -> bool;
    fn save(&self, task: &Task) -> Result<()>;
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
