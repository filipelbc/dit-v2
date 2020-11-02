use chrono::{DateTime, Local};

pub type LocalDateTime = DateTime<Local>;

pub fn resolve(at: Option<&str>) -> LocalDateTime {
    match at {
        Some(x) => parse(x),
        None => Local::now(),
    }
}

fn parse(x: &str) -> LocalDateTime {
    Local::now()
}
