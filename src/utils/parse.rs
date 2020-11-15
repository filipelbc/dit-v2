use anyhow::{Context, Result};
use std::str::FromStr;

pub fn parse_usize(s: &str) -> Result<usize> {
    usize::from_str(s).with_context(|| format!("Invalid number: {}", s))
}
