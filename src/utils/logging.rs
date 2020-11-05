use log::{LevelFilter, Metadata, Record};

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{}", record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init(verbosity: u64) {
    log::set_boxed_logger(Box::new(Logger)).unwrap();

    match verbosity {
        0 => log::set_max_level(LevelFilter::Info),
        1 => log::set_max_level(LevelFilter::Debug),
        _ => log::set_max_level(LevelFilter::Trace),
    }
}
