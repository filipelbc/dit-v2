use anyhow::{Context, Result};
use dialoguer::Input;

pub fn prompt(p: &str) -> Result<String> {
    Input::new()
        .with_prompt(p)
        .allow_empty(true)
        .validate_with(|x: &String| {
            if x.trim().is_empty() {
                Err(format!("{} cannot be empty", p))
            } else {
                Ok(())
            }
        })
        .interact_text()
        .map(|x| x.trim().to_string())
        .context("Could not read task title")
}
