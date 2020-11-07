use anyhow::{bail, Context, Result};
use dirs::home_dir;
use log::trace;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DIT_DIR_NAME: &str = ".dit";

pub fn resolve(path: Option<&str>) -> Result<PathBuf> {
    let directory = match path {
        Some(path) => ensure_exists(Path::new(path)).map(|_| PathBuf::from(path)),
        None => {
            let path = env::current_dir().context("Could not read current directory")?;
            search_from(path)
        }
    };

    directory.and_then(|p| {
        p.canonicalize()
            .with_context(|| format!("Could not canonicalize directory: {}", p.display()))
    })
}

pub fn ensure_exists(path: &Path) -> Result<()> {
    if path.exists() {
        return match path.is_dir() {
            true => Ok(()),
            false => bail!("Path exists but is not a directory: {}", path.display()),
        };
    }

    trace!("Creating directory: {}", path.display());
    fs::create_dir_all(&path)
        .with_context(|| format!("Could not create directory: {}", path.display()))
}

fn search_from(path: PathBuf) -> Result<PathBuf> {
    let mut ancerstors = path.ancestors();
    while let Some(p) = ancerstors.next() {
        let path = p.join(DIT_DIR_NAME);
        if path.is_dir() {
            return Ok(path);
        }
    }
    let home = home_dir().context("Could not find home directory")?;
    let p = home.join(DIT_DIR_NAME);
    ensure_exists(p.as_path()).map(|_| p)
}
