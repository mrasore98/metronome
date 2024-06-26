use clap::builder::NonEmptyStringValueParser;
use clap::{command, Arg, ArgAction, ArgMatches, Command};

pub fn match_cli() -> ArgMatches {
    let matches = command!()
        .args_conflicts_with_subcommands(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("start")
                .about("Start a new task.")
                .arg(
                    Arg::new("task")
                        .help("Name of the task to start")
                        .value_parser(NonEmptyStringValueParser::new())
                        .required(true),
                )
                .arg(
                    Arg::new("category")
                        .help("Specify a category for the new task.")
                        .short('c')
                        .long("category")
                        .value_parser(NonEmptyStringValueParser::new()),
                ),
        )
        .subcommand(
            Command::new("end")
                .about("End an existing task.")
                .arg(
                    Arg::new("task")
                        .help("Name of the task to end.")
                        .value_parser(NonEmptyStringValueParser::new())
                        .required_unless_present_any(["all", "last"])
                        .exclusive(true),
                )
                .arg(
                    Arg::new("last")
                        .short('l')
                        .long("last")
                        .help("Ends the active task that was started most recently.")
                        .action(ArgAction::SetTrue)
                        .exclusive(true),
                )
                .arg(
                    Arg::new("all")
                        .long("all")
                        .help("Ends all active tasks. Overrides a task name if one is given.")
                        .action(ArgAction::SetTrue)
                        .exclusive(true),
                ),
        )
        .subcommand(
            Command::new("list")
                .about("Display a list of tasks.")
                .arg(
                    Arg::new("active")
                        .short('a')
                        .long("active")
                        .help("List the active tasks.")
                        .action(ArgAction::SetTrue)
                        .conflicts_with_all(["all", "completed"]),
                )
                .arg(
                    Arg::new("completed")
                        .help("List the completed tasks.")
                        .short('c')
                        .long("complete")
                        .alias("completed")
                        .action(ArgAction::SetTrue)
                        .conflicts_with_all(["active", "all"]),
                )
                .arg(
                    Arg::new("all")
                        .help("List all tasks.")
                        .long("all")
                        .action(ArgAction::SetTrue)
                        .conflicts_with_all(["active", "completed"]),
                )
                .arg(
                    Arg::new("filter")
                        .help("Apply a time range filter to the list of tasks.")
                        .short('f')
                        .long("filter")
                        // see src/core/filters.rs
                        .value_parser([
                            "d",
                            "day",
                            "w",
                            "week",
                            "m",
                            "month",
                            "q",
                            "quarter",
                            "s",
                            "semi",
                            "semiannual",
                            "y",
                            "year",
                        ]),
                ),
        )
        .subcommand(
            Command::new("total")
                .about("Sum the amount of time spent on your tasks.")
                .arg(
                    Arg::new("filter")
                        .help("Only total tasks within the time range specified by a filter.")
                        .short('f')
                        .long("filter")
                        .value_parser([
                            "d",
                            "day",
                            "w",
                            "week",
                            "m",
                            "month",
                            "q",
                            "quarter",
                            "s",
                            "semi",
                            "semiannual",
                            "y",
                            "year",
                        ]),
                )
                .arg(
                    Arg::new("category")
                        .help("Only total tasks in specified categories.")
                        .short('c')
                        .long("category")
                        .value_parser(NonEmptyStringValueParser::new()),
                ),
        )
        .get_matches();

    matches
}
