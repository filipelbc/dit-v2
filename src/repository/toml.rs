use std::path::PathBuf;

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
        self.directory.join(id).exists()
    }

    fn save(&self, task: &Task) -> Result<(), String> {
        Err(String::from("Not implemented"))
    }
}
