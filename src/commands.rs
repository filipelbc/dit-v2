use anyhow::{bail, Result};
use log::info;

use crate::models::{Repository, Task};
use crate::repository::toml::Repo;
use crate::utils::time::LocalDateTime;

pub struct Dit {
    pub repo: Repo,
}

impl Dit {
    pub fn new(repo: Repo) -> Self {
        Dit { repo }
    }

    pub fn do_new(&self, key: &str, title: Option<&str>, fetch: bool) -> Result<()> {
        let id = self.repo.resolve_key(key);

        if self.repo.exists(&id) {
            bail!("Task already exists: {}", id);
        }

        let task = Task::new(id);

        self.repo.save(&task)
            .map(|()| info!("Created: {}", task.id))
    }

    pub fn do_work_on(&self, key: &str, now: LocalDateTime) -> Result<()> {
        let id = self.repo.resolve_key(key);

        if !self.repo.exists(&id) {
            bail!("Task does not exist: {}", id);
        }

        bail!("Not implemented")
    }

    pub fn do_halt(&self, now: LocalDateTime) -> Result<()> {
        bail!("Not implemented")
    }

    pub fn do_append(&self) -> Result<()> {
        bail!("Not implemented")
    }

    pub fn do_cancel(&self) -> Result<()> {
        bail!("Not implemented")
    }

    pub fn do_resume(&self, now: LocalDateTime) -> Result<()> {
        bail!("Not implemented")
    }

    pub fn do_switch_back(&self, now: LocalDateTime) -> Result<()> {
        bail!("Not implemented")
    }
}
