use dirs::home_dir;
use log::trace;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DIT_DIR_NAME: &str = ".dit";

pub fn resolve(path: Option<&str>) -> Result<PathBuf, String> {
    let directory = match path {
        Some(path) => ensure_exists(Path::new(path)).map(|_| PathBuf::from(path)),
        None => match env::current_dir() {
            Ok(path) => search_from(path),
            Err(e) => Err(format!("Could not read current directory: {}", e)),
        },
    };

    directory.and_then(|p| {
        p.canonicalize().map_err(|e| {
            format!(
                "Could not canonicalize directory: {}\n-> {}",
                p.display(),
                e
            )
        })
    })
}

pub fn ensure_exists(path: &Path) -> Result<(), String> {
    match path.exists() {
        true => match path.is_dir() {
            true => Ok(()),
            false => Err(format!("Not a directory: {}", path.display())),
        },
        false => {
            trace!("Creating directory: {}", path.display());
            fs::create_dir_all(&path)
                .map_err(|e| format!("Could not create directory: {}\n-> {}", path.display(), e))
        }
    }
}

fn search_from(path: PathBuf) -> Result<PathBuf, String> {
    let mut ancerstors = path.ancestors();
    while let Some(p) = ancerstors.next() {
        let path = p.join(DIT_DIR_NAME);
        if path.is_dir() {
            return Ok(path);
        }
    }
    match home_dir() {
        Some(home) => {
            let p = home.join(DIT_DIR_NAME);
            ensure_exists(p.as_path()).map(|_| p)
        }
        None => Err(String::from("Home directory not found")),
    }
}
