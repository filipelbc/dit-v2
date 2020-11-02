use crate::utils::time::LocalDateTime;
use crate::repository::toml::Repo;

pub struct Dit {
    pub repo: Repo,
}

impl Dit {
    pub fn new(repo: Repo) -> Self {
        Dit {
            repo
        }
    }

    pub fn do_new(&self, task: &str, title: Option<&str>, fetch: bool) -> Result<(), String> {
        Err(String::from("Not implemented"))
    }

    pub fn do_work_on(&self, task: &str, now: LocalDateTime) -> Result<(), String> {
        Err(String::from("Not implemented"))
    }

    pub fn do_halt(&self, now: LocalDateTime) -> Result<(), String> {
        Err(String::from("Not implemented"))
    }

    pub fn do_append(&self) -> Result<(), String> {
        Err(String::from("Not implemented"))
    }

    pub fn do_cancel(&self) -> Result<(), String> {
        Err(String::from("Not implemented"))
    }

    pub fn do_resume(&self, now: LocalDateTime) -> Result<(), String> {
        Err(String::from("Not implemented"))
    }

    pub fn do_switch_back(&self, now: LocalDateTime) -> Result<(), String> {
        Err(String::from("Not implemented"))
    }
}
