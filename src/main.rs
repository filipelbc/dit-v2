use anyhow::{bail, Context, Result};
use clap::ArgMatches;
use log::debug;

mod utils;
use crate::utils::graceful::Graceful;
use crate::utils::time::{Timestamp, parse_timestamp, now};

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

            dit.do_resume(now)
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

            dit.do_switch_back(now)
        }
        Some(("status", cargs)) => {
            let limit = get_usize(cargs, "limit")?;
            let rebuild = cargs.is_present("rebuild-index");
            let short = cargs.is_present("short");
            dit.do_status(limit, rebuild, short)
        }
        Some(("list", cargs)) => dit.do_list(
            cargs.is_present("daily"),
            cargs.is_present("daily-only"),
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

    run(args).graceful();
}
