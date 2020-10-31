use clap::{App, Arg};
use log::debug;
use std::env;

mod utils;
use utils::graceful::Graceful;

mod models;

fn fetch_arg<'a>() -> Arg<'a> {
    Arg::new("fetch")
        .short('f')
        .long("fetch")
        .about("Use data fetcher plugin.")
}

fn task_param<'a>() -> Arg<'a> {
    Arg::new("task")
        .about("The target task for the command.")
        .value_name("TASK")
        .required(true)
        .validator(models::validate_task_key)
}

fn new_arg<'a>() -> Arg<'a> {
    Arg::new("new")
        .about("Also create the task.")
        .long("new")
        .short('n')
}

fn title_arg<'a>() -> Arg<'a> {
    Arg::new("title")
        .about("Title of the new task. Only relevant if '--new' is used. If absent, you'll be prompted for it.")
        .value_name("TITLE")
        .requires("new")
}

fn at_arg<'a>() -> Arg<'a> {
    Arg::new("at")
        .about("Use the given datetime instead of 'now'.")
        .value_name("DATETIME")
        .long("at")
        .short('a')
}

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("directory")
            .about("Sets the dit data directory. If not specified, the closest '.dit' directory in the tree is used. If none is found, '~/.dit' is used.")
            .long("directory")
            .short('d')
            .value_name("DIRECTORY")
            .takes_value(true)
        )
        .arg(
            Arg::new("verbose")
            .about("Prints detailed information of what is being done.")
            .long("verbose")
            .short('v')
        )
        .arg(
            Arg::new("check-hooks")
            .about("Stop with error if a hook process fail.")
            .long("check-hooks")
        )
        .arg(
            Arg::new("no-hooks")
            .about("Disables the use of hooks.")
            .long("no-hooks")
        )
        .subcommand(
            App::new("new")
            .visible_alias("n")
            .about("Creates a new task. You'll be prompted for its title if it is not provided.")
            .arg(
                Arg::new("task")
                .about("Task to be created. This is the main identifier of the task. Use '/' to create nested tasks, e.g. 'foo/bar'.")
                .value_name("TASK")
                .required(true)
                .validator(models::validate_task_key)
            )
            .arg(
                Arg::new("title")
                .about("Title of the task. If absent, you'll be prompted for it.")
                .value_name("TITLE")
                .required(false)
            )
            .arg(fetch_arg())
        )
        .subcommand(
            App::new("work-on")
            .visible_alias("w")
            .about("Starts clocking on the specified task. Does nothing if there already is an active task. Sets the CURRENT task.")
            .arg(task_param())
            .arg(at_arg())
            .arg(new_arg())
            .arg(fetch_arg())
            .arg(title_arg()),
        )
        .subcommand(
            App::new("halt")
            .visible_alias("h")
            .about("Stops clocking on the currently active task. Does nothing if there is no active task.")
            .arg(at_arg())
        )
        .subcommand(
            App::new("append")
            .visible_alias("a")
            .about("Undoes the previous 'halt'.")
        )
        .subcommand(
            App::new("cancel")
            .visible_alias("c")
            .about("Undoes the previous 'work-on'. Does nothing if there is no active task.")
        )
        .subcommand(
            App::new("resume")
            .visible_alias("r")
            .about("Starts clocking on the CURRENT task. Same as a 'work-on CURRENT'. Does nothing if there is no CURRENT task.")
            .arg(at_arg())
        )
        .subcommand(
            App::new("switch-to")
            .visible_alias("s")
            .about("Stops clocking on the CURRENT task, and starts clocking on the specified task. Same as 'halt' followed by 'work-on TASK'.")
            .arg(task_param())
            .arg(at_arg())
            .arg(new_arg())
            .arg(fetch_arg())
            .arg(title_arg())
        )
        .subcommand(
            App::new("switch-back")
            .visible_alias("b")
            .about("Stops clocking on the CURRENT task, and starts clocking on the PREVIOUS task. Same as 'halt' followed by 'work-on PREVIOUS'.")
            .arg(at_arg())
        )
        .get_matches();

    utils::logging::init(matches.is_present("verbose"));

    let directory = utils::directory::resolve(matches.value_of("directory")).graceful();
    debug!("Using data directory: {}", directory.display());
}
