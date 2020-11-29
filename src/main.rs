use anyhow::{bail, Context, Result};
use clap::ArgMatches;
use log::{debug, error};
use std::process::exit;
use std::str::FromStr;

mod utils;
use crate::utils::time::{now, parse_timestamp, Timestamp};

mod models;

mod repository;
use crate::repository::toml::Repo;

mod commands;
use crate::commands::Dit;

mod cli;

fn get_usize(cargs: &ArgMatches, name: &str) -> Result<usize> {
    let s = cargs.value_of(name).unwrap();
    usize::from_str_radix(s, 10).with_context(|| format!("Invalid value for '{}': {}", name, s))
}

fn get_timestamp(cargs: &ArgMatches, name: &str) -> Result<Option<Timestamp>> {
    match cargs.value_of(name) {
        Some(x) => parse_timestamp(x)
            .with_context(|| format!("Invalid date/time value for '{}': {}", name, x))
            .map(Some),
        None => Ok(None),
    }
}

fn get_at(cargs: &ArgMatches) -> Result<Timestamp> {
    get_timestamp(cargs, "at").map(|x| x.unwrap_or_else(|| now()))
}

fn get_single<T>(cargs: &ArgMatches, name: &str) -> Result<T, T::Err>
where
    T: FromStr,
{
    T::from_str(cargs.value_of(name).unwrap())
}

fn get_many<T>(cargs: &ArgMatches, name: &str) -> Result<Vec<T>, T::Err>
where
    T: FromStr,
{
    cargs.values_of(name).unwrap().map(T::from_str).collect()
}

fn run(args: ArgMatches) -> Result<()> {
    let directory = utils::directory::resolve(args.value_of("directory"))?;
    debug!("Using data directory: {}", directory.display());

    let repo = Repo::new(directory)?;
    let dit = Dit::new(Box::new(repo));

    match args.subcommand() {
        Some(("new", cargs)) => dit.do_new(
            cargs.value_of("task").unwrap(),
            cargs.value_of("title"),
            cargs.is_present("fetch"),
        ),
        Some(("work-on", cargs)) => {
            let task = cargs.value_of("task").unwrap();
            let now = get_at(&cargs)?;

            if cargs.is_present("new") {
                dit.do_new(task, cargs.value_of("title"), cargs.is_present("fetch"))?;
            }

            dit.do_work_on(task, now)
        }
        Some(("halt", cargs)) => {
            let now = get_at(&cargs)?;

            dit.do_halt(now)
        }
        Some(("append", _)) => dit.do_append(),
        Some(("cancel", _)) => dit.do_cancel(),
        Some(("resume", cargs)) => {
            let now = get_at(&cargs)?;

            dit.do_work_on_by_index(now, get_usize(cargs, "index")?)
        }
        Some(("switch-to", cargs)) => {
            let task = cargs.value_of("task").unwrap();
            let now = get_at(&cargs)?;

            if cargs.is_present("new") {
                dit.do_new(task, cargs.value_of("title"), cargs.is_present("fetch"))?;
            }

            dit.do_halt(now)?;

            dit.do_work_on(task, now)
        }
        Some(("switch-back", cargs)) => {
            let now = get_at(&cargs)?;

            dit.do_halt(now)?;

            dit.do_work_on_by_index(now, get_usize(cargs, "index")?)
        }
        Some(("status", cargs)) => dit.do_status(
            get_usize(cargs, "limit")?,
            cargs.is_present("rebuild-index"),
            cargs.is_present("short"),
        ),
        Some(("list", cargs)) => dit.do_list(
            get_single(cargs, "mode")?,
            get_single(cargs, "format")?,
            get_timestamp(&cargs, "after")?,
            get_timestamp(&cargs, "before")?,
        ),
        Some((cmd, _)) => bail!("Unhandled subcommand: {}", cmd),
        None => bail!("No subcommand provided"),
    }
}

fn main() {
    let args = cli::parse();

    utils::logging::init(args.occurrences_of("verbose"));

    match run(args) {
        Err(err) => {
            error!("{:?}", err);
            exit(1);
        }
        _ => (),
    }
}
