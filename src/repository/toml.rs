use std::path::PathBuf;

use crate::models::Repository;

pub struct Repo {
    directory: PathBuf,
}

impl Repository for Repo {
    fn new(directory: PathBuf) -> Self {
        Repo { directory }
    }
}
