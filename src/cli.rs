use clap::{App, AppSettings, Arg, ArgMatches};

use crate::models::Task;

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
        .validator(Task::validate_key)
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
        .allow_hyphen_values(true)
}

fn new_app<'a>(name: &str) -> App<'a> {
    App::new(name).setting(AppSettings::UnifiedHelpMessage)
}

pub fn parse() -> ArgMatches {
    new_app(env!("CARGO_PKG_NAME"))
        .setting(AppSettings::SubcommandRequired)
        .setting(AppSettings::DeriveDisplayOrder)
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
            .global(true)
        )
        .arg(
            Arg::new("verbose")
            .about("Prints detailed information of what is being done.")
            .long("verbose")
            .short('v')
            .global(true)
            .multiple_occurrences(true)
        )
        .arg(
            Arg::new("check-hooks")
            .about("Stop with error if a hook process fail.")
            .long("check-hooks")
            .global(true)
        )
        .arg(
            Arg::new("no-hooks")
            .about("Disables the use of hooks.")
            .long("no-hooks")
            .global(true)
        )
        .subcommand(
            new_app("new")
            .visible_alias("n")
            .about("Creates a new task. You'll be prompted for a title if one is not provided.")
            .arg(
                Arg::new("task")
                .about("Task to be created. This is the main identifier of the task. Use '/' to create nested tasks, e.g. 'foo/bar'.")
                .value_name("TASK")
                .required(true)
                .validator(Task::validate_key)
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
            new_app("work-on")
            .visible_alias("w")
            .about("Starts clocking on the specified task.")
            .arg(task_param())
            .arg(at_arg())
            .arg(new_arg())
            .arg(fetch_arg())
            .arg(title_arg()),
        )
        .subcommand(
            new_app("halt")
            .visible_alias("h")
            .about("Stops clocking on the currently active task.")
            .arg(at_arg())
        )
        .subcommand(
            new_app("append")
            .visible_alias("a")
            .about("Undoes the previous 'halt'.")
        )
        .subcommand(
            new_app("cancel")
            .visible_alias("c")
            .about("Undoes the previous 'work-on'.")
        )
        .subcommand(
            new_app("resume")
            .visible_alias("r")
            .about("Starts clocking on the last active task.")
            .arg(at_arg())
            .arg(
                Arg::new("index")
                    .about("Resumes working on the the I'th previous task.")
                    .value_name("I")
                    .default_value("0")
            )
        )
        .subcommand(
            new_app("switch-to")
            .visible_alias("t")
            .about("Stops clocking on the currently active task, and starts clocking on the specified task.")
            .arg(task_param())
            .arg(at_arg())
            .arg(new_arg())
            .arg(fetch_arg())
            .arg(title_arg())
        )
        .subcommand(
            new_app("switch-back")
            .visible_alias("b")
            .about("Stops clocking on the currently active task, and starts clocking on the previous task.")
            .arg(at_arg())
            .arg(
                Arg::new("index")
                    .about("Switches back to the ith previous task.")
                    .value_name("I")
                    .default_value("1")
            )
        )
        .subcommand(
            new_app("status")
            .visible_alias("s")
            .about("Prints the most recent tasks.")
            .arg(
                Arg::new("limit")
                    .about("Limits listing to last NUM tasks.")
                    .value_name("NUM")
                    .long("limit")
                    .short('n')
                    .default_value("10")
            )
            .arg(
                Arg::new("rebuild-index")
                    .about("Rebuilds the task index before printing the status.")
                    .long("rebuild-index")
                    .short('r')
            )
            .arg(
                Arg::new("short")
                    .about("Prints just the current task plus the duration for which it has been active.")
                    .long("short")
                    .short('s')
            )
        )
        .subcommand(
            new_app("list")
            .visible_alias("l")
            .about("Lists log entries in chronological order, most recent first")
            .arg(
                Arg::new("daily")
                    .about("Show daily summaries")
                    .long("daily")
            )
            .arg(
                Arg::new("daily-only")
                    .about("Show only daily summaries")
                    .long("daily-only")
            )
            .arg(
                Arg::new("after")
                    .about("Consider only entries from after this date")
                    .value_name("DATETIME")
                    .long("after")
            )
            .arg(
                Arg::new("before")
                    .about("Consider only entries from before this date")
                    .value_name("DATETIME")
                    .long("before")
            )
        )
        .get_matches()
}
