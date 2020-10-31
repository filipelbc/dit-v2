use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref TASK_KEY: Regex = Regex::new("^(/?[A-Za-z][0-9A-Za-z_-]*)+$").unwrap();
}

pub fn validate_task_key(key: &str) -> Result<(), String> {
    match TASK_KEY.is_match(key) {
        true => Ok(()),
        false => Err(String::from(key)),
    }
}
