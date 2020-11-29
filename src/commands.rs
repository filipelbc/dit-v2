use anyhow::{bail, Result};
use chrono::{Date, Duration, FixedOffset};
use log::{debug, info};
use std::str::FromStr;

use crate::models::{ListItem, Repository, StatusItem, Task};
use crate::utils::input::prompt;
use crate::utils::nice::Nice;
use crate::utils::tables::{Column, Table};
use crate::utils::time::Timestamp;

macro_rules! columns {
    ($t:ty, $($c:pat => $n:expr, $x:expr),+ $(,)?) => {
        |p| match p {
            $(
                $c => { Column::<$t>::new($n, $x) },
            )+
        }
    };
}

pub enum StatusProperties {
    Id,
    Title,
    Start,
    End,
    Effort,
    TotalEffort,
}

pub enum ListProperties {
    Id,
    Title,
    Start,
    End,
    Effort,
}

pub enum ListMode {
    GroupByDay,
    Plain,
    Daily,
}

pub enum ListFormat {
    Table,
    JsonLines,
    Csv,
}

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
            bail!("Already working on a task: {}", task_id);
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
        if let Some((id, entry)) = self.repo.previous_task(0) {
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

    pub fn do_work_on_by_index(&self, now: Timestamp, index: usize) -> Result<()> {
        if let Some((id, entry)) = self.repo.previous_task(index) {
            if entry.is_closed() {
                return self
                    .repo
                    .clock_in(&id, now)
                    .map(|()| info!("Working on: {}", id));
            }
            bail!("Already working on a task: {}", id);
        }
        bail!("No previous task {} to work on; rebuild index?", index);
    }

    pub fn do_status(
        &self,
        short: bool,
        rebuild: bool,
        limit: usize,
        properties: &[StatusProperties],
    ) -> Result<()> {
        if rebuild {
            debug!("Rebuilding index");
            self.repo.rebuild_index()?;
            debug!("Done")
        }

        let status = self.repo.get_status(limit);

        if short {
            if let Some(s) = status.first() {
                if s.log_entry.is_open() {
                    println!("{} {}", s.id, s.effort().nice());
                }
            }
        } else {
            let t = Table::new(
                properties
                    .iter()
                    .map(columns!(StatusItem,
                        StatusProperties::Id          => "Id",          |x| x.id.to_string(),
                        StatusProperties::Title       => "Title",       |x| x.title.to_string(),
                        StatusProperties::Start       => "Start",       |x| x.start().nice(),
                        StatusProperties::End         => "End",         |x| x.end().nice(),
                        StatusProperties::Effort      => "Effort",      |x| x.effort().nice(),
                        StatusProperties::TotalEffort => "TotalEffort", |x| x.total_effort.nice(),
                    ))
                    .collect(),
            );

            t.print(&status);
        }

        Ok(())
    }

    pub fn do_list(
        &self,
        mode: ListMode,
        format: ListFormat,
        properties: &[ListProperties],
        after: Option<Timestamp>,
        before: Option<Timestamp>,
    ) -> Result<()> {
        let data = self.repo.get_listing(after, before)?;

        let t = Table::new(
            properties
                .iter()
                .map(columns!(ListItem,
                    ListProperties::Id     => "Id",     |x| x.id.to_string(),
                    ListProperties::Title  => "Title",  |x| x.title.to_string(),
                    ListProperties::Start  => "Start",  |x| x.start().nice(),
                    ListProperties::End    => "End",    |x| x.end().nice(),
                    ListProperties::Effort => "Effort", |x| x.effort().nice(),
                ))
                .collect(),
        );

        match mode {
            ListMode::GroupByDay => {
                for (key, items) in group_by_day(&data) {
                    println!("{}: {}", key.nice(), total_effort(items).nice());
                    t.print(&items);
                }
            }
            ListMode::Daily => {
                for (key, items) in group_by_day(&data) {
                    println!("{}: {}", key.nice(), total_effort(items).nice());
                }
            }
            ListMode::Plain => t.print(&data),
        }

        Ok(())
    }
}

fn total_effort(x: &[ListItem]) -> Duration {
    x.iter().fold(Duration::seconds(0), |a, x| a + x.effort())
}

// based on: https://stackoverflow.com/a/50392400
fn group_by_day(x: &[ListItem]) -> impl Iterator<Item = (Date<FixedOffset>, &[ListItem])> {
    let key = |z: &ListItem| z.log_entry.start.date();

    let mut slice_start = 0;

    (1..x.len() + 1).flat_map(move |i| {
        let k = key(&x[i - 1]);

        if i == x.len() || k != key(&x[i]) {
            let start = slice_start;
            slice_start = i;
            Some((k, &x[start..i]))
        } else {
            None
        }
    })
}

impl FromStr for StatusProperties {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "id" => Ok(Self::Id),
            "title" => Ok(Self::Title),
            "start" => Ok(Self::Start),
            "end" => Ok(Self::End),
            "effort" => Ok(Self::Effort),
            "total-effort" => Ok(Self::TotalEffort),
            _ => bail!("Invalid task field: {}", s),
        }
    }
}

impl FromStr for ListProperties {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "id" => Ok(Self::Id),
            "title" => Ok(Self::Title),
            "start" => Ok(Self::Start),
            "end" => Ok(Self::End),
            "effort" => Ok(Self::Effort),
            _ => bail!("Invalid task field: {}", s),
        }
    }
}

impl FromStr for ListMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "group-by-day" => Ok(Self::GroupByDay),
            "plain" => Ok(Self::Plain),
            "daily" => Ok(Self::Daily),
            _ => bail!("Invalid list mode: {}", s),
        }
    }
}

impl FromStr for ListFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "table" => Ok(Self::Table),
            "json-lines" => Ok(Self::JsonLines),
            "csv" => Ok(Self::Csv),
            _ => bail!("Invalid list format: {}", s),
        }
    }
}
