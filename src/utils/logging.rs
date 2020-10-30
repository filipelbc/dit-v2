use log::{Level, LevelFilter, Metadata, Record};

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{}", record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init(verbose: bool) {
    log::set_boxed_logger(Box::new(Logger)).unwrap();

    match verbose {
        false => log::set_max_level(LevelFilter::Info),
        true => log::set_max_level(LevelFilter::Debug),
    }
}
