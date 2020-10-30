use log::debug;
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn resolve(path: Option<&str>) -> Result<PathBuf, String> {
    println!("{:?}", path);
    let directory = match path {
        Some(path) => ensure_exists(PathBuf::from(path)),
        None => match env::current_dir() {
            Ok(path) => search_from(path),
            _ => Err(format!("Could not read current directory")),
        },
    };

    match directory {
        Ok(path) => match path.canonicalize() {
            Ok(path) => Ok(path),
            _ => Err(format!(
                "Could not canonicalize directory: {}",
                path.display()
            )),
        },
        Err(x) => Err(x),
    }
}

fn ensure_exists(path: PathBuf) -> Result<PathBuf, String> {
    match path.exists() {
        true => match path.is_dir() {
            true => Ok(path),
            false => Err(format!("Not a directory: {}", path.display())),
        },
        false => match fs::create_dir_all(&path) {
            Ok(()) => {
                debug!("Created directory: {}", path.display());
                Ok(path)
            }
            _ => Err(format!("Could not create directory: {}", path.display())),
        },
    }
}

fn search_from(path: PathBuf) -> Result<PathBuf, String> {
    let mut ancerstors = path.ancestors();
    while let Some(p) = ancerstors.next() {
        let path = p.join(".dit");
        if path.is_dir() {
            return Ok(path);
        }
    }
    ensure_exists(PathBuf::from("~/.dit"))
}
