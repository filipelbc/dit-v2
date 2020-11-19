use anyhow::{bail, Result};
use log::info;

use crate::models::{Repository, Status, Task};
use crate::utils::input::prompt;
use crate::utils::tables::{Column, Table};
use crate::utils::time::{format_duration, format_timestamp, Timestamp};
use crate::table;

pub struct Dit {
    pub repo: Box<dyn Repository>,
}

impl Dit {
    pub fn new(repo: Box<dyn Repository>) -> Self {
        Dit { repo }
    }

    pub fn do_new(&self, key: &str, title: Option<&str>, fetch: bool) -> Result<()> {
        let id = self.repo.resolve_key(key);

        if self.repo.exists(&id) {
            bail!("Task already exists: {}", id);
        }

        let mut task = Task::new(id);

        task.data.title = match title {
            Some(t) => t.to_string(),
            None => prompt("Title")?,
        };

        self.repo
            .save(&task)
            .map(|()| info!("Created: {}", task.id))
    }

    pub fn do_work_on(&self, key: &str, now: Timestamp) -> Result<()> {
        let id = self.repo.resolve_key(key);

        if !self.repo.exists(&id) {
            bail!("Task does not exist: {}", id);
        }

        if let Some(task_id) = self.repo.is_clocked_in() {
            bail!("Already working on: {}", task_id);
        }

        self.repo
            .clock_in(&id, now)
            .map(|()| info!("Working on: {}", id))
    }

    pub fn do_halt(&self, now: Timestamp) -> Result<()> {
        if let Some(id) = self.repo.is_clocked_in() {
            return self
                .repo
                .clock_out(&id, now)
                .map(|()| info!("Halted: {}", id));
        }
        bail!("Not working on any task");
    }

    pub fn do_append(&self) -> Result<()> {
        if let Some((id, entry)) = self.repo.current_task() {
            if entry.is_closed() {
                return self
                    .repo
                    .un_clock_out(&id)
                    .map(|()| info!("Appending to: {}", id));
            }
            bail!("Already working on: {}", id);
        }
        bail!("No previous task to append to; rebuild index?")
    }

    pub fn do_cancel(&self) -> Result<()> {
        if let Some(id) = self.repo.is_clocked_in() {
            return self
                .repo
                .un_clock_in(&id)
                .map(|()| info!("Canceled: {}", id));
        }
        bail!("Not working on any task");
    }

    pub fn do_resume(&self, now: Timestamp) -> Result<()> {
        if let Some((id, entry)) = self.repo.current_task() {
            if entry.is_closed() {
                return self
                    .repo
                    .clock_in(&id, now)
                    .map(|()| info!("Resuming: {}", id));
            }
            bail!("Already working on: {}", id);
        }
        bail!("No previous task to resume; rebuild index?")
    }

    pub fn do_switch_back(&self, now: Timestamp) -> Result<()> {
        bail!("Not implemented")
    }

    pub fn do_status(&self, limit: usize, rebuild: bool, short: bool) -> Result<()> {
        let status = self.repo.get_status(limit);

        let t = table![
            Status,
            "Start",
            |x| format_timestamp(&x.start()),
            "End",
            |x| format_timestamp(&x.end()),
            "Effort",
            |x| format_duration(&x.duration()),
            "Total Effort",
            |x| format_duration(&x.time_spent),
            "Id",
            |x| x.id.to_string(),
            "Title",
            |x| x.title.to_string(),
        ];
        t.print(&status);

        Ok(())
    }
}
