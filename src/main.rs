use anyhow::{bail, Result};
use clap::ArgMatches;
use log::debug;

mod utils;
use crate::utils::graceful::Graceful;
use crate::utils::parse::parse_usize;

mod models;

mod repository;
use crate::repository::toml::Repo;

mod commands;
use crate::commands::Dit;

mod cli;

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
            let now = utils::time::resolve(cargs.value_of("at"))?;

            if cargs.is_present("new") {
                dit.do_new(task, cargs.value_of("title"), cargs.is_present("fetch"))?;
            }

            dit.do_work_on(task, now)
        }
        Some(("halt", cargs)) => {
            let now = utils::time::resolve(cargs.value_of("at"))?;

            dit.do_halt(now)
        }
        Some(("append", _)) => dit.do_append(),
        Some(("cancel", _)) => dit.do_cancel(),
        Some(("resume", cargs)) => {
            let now = utils::time::resolve(cargs.value_of("at"))?;

            dit.do_resume(now)
        }
        Some(("switch-to", cargs)) => {
            let task = cargs.value_of("task").unwrap();
            let now = utils::time::resolve(cargs.value_of("at"))?;

            if cargs.is_present("new") {
                dit.do_new(task, cargs.value_of("title"), cargs.is_present("fetch"))?;
            }

            dit.do_halt(now)?;

            dit.do_work_on(task, now)
        }
        Some(("switch-back", cargs)) => {
            let now = utils::time::resolve(cargs.value_of("at"))?;

            dit.do_switch_back(now)
        }
        Some(("status", cargs)) => {
            let limit = parse_usize(cargs.value_of("limit").unwrap_or("0"))?;
            let rebuild = cargs.is_present("rebuild-index");
            let short = cargs.is_present("short");
            dit.do_status(limit, rebuild, short)
        }
        Some((cmd, _)) => bail!("Unhandled subcommand: {}", cmd),
        None => bail!("No subcommand provided"),
    }
}

fn main() {
    let args = cli::parse();

    utils::logging::init(args.occurrences_of("verbose"));

    run(args).graceful();
}
