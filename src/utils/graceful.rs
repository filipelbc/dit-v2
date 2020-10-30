use log::error;
use std::fmt::Display;
use std::process;

pub trait Graceful<T, E: Display> {
    fn graceful(self) -> T;
}

impl<T, E: Display> Graceful<T, E> for Result<T, E> {
    fn graceful(self) -> T {
        match self {
            Ok(val) => val,
            Err(err) => {
                error!("{}", err);
                process::exit(1);
            }
        }
    }
}
