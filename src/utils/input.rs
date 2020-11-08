use anyhow::{Context, Result};
use dialoguer::Input;

pub fn prompt(p: &str) -> Result<String> {
    Input::new()
        .with_prompt(p)
        .interact_text()
        .context("Could not read task title")
}
