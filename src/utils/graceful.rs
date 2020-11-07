use log::error;
use std::fmt::Debug;
use std::process;

pub trait Graceful<T, E: Debug> {
    fn graceful(self) -> T;
}

impl<T, E: Debug> Graceful<T, E> for Result<T, E> {
    fn graceful(self) -> T {
        match self {
            Ok(val) => val,
            Err(err) => {
                error!("{:?}", err);
                process::exit(1);
            }
        }
    }
}
