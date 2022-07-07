use clap::{Arg, ArgGroup, Command};

use crate::tag::{command_from_tag, Tags};

const ABOUT: &str = "
opentag (ot) opens a tagged path or URL using the configured system program.

Tags are defined in a `json` data file. See the project home page for
information about the structure and the location of this file.

The tags are added to the application as \"subcommands\" at run-time and appear
in the help text.

Project home page: https://github.com/nextonesfaster/opentag
";

const HELP_TEMPLATE: &str = "{before-help}{bin} {version}
{author}

{about}

{usage-heading}
    ot <--add|--remove|--update|--list>
    ot [OPTIONS|--list] <TAG>

{all-args}{after-help}";

pub fn create_tags_app(tags: &Tags) -> Command {
    clap::command!()
        .arg_required_else_help(true)
        .subcommand_negates_reqs(true)
        .disable_help_subcommand(true)
        .about(ABOUT.trim_start().lines().next())
        .long_about(ABOUT)
        .help_template(HELP_TEMPLATE)
        .hide_possible_values(true)
        .subcommand_help_heading("TAGS")
        .arg(
            Arg::new("print")
                .short('p')
                .long("print")
                .global(true)
                .help("Print the path or the URL instead of opening it."),
        )
        .arg(
            Arg::new("app")
                .short('A')
                .long("app")
                .takes_value(true)
                .conflicts_with_all(&["print", "silent-copy"])
                .global(true)
                .help("Specify the app to open the path or the URL with."),
        )
        .arg(
            Arg::new("copy")
                .short('c')
                .long("copy")
                .global(true)
                .help("Copy the path or the URL to the system's clipboard."),
        )
        .arg(
            Arg::new("silent-copy")
                .short('C')
                .long("silent-copy")
                .global(true)
                .help(
                    "Copy the path or the URL to the system's clipboard without opening the path.",
                ),
        )
        .arg(
            Arg::new("add")
                .short('a')
                .long("add")
                .help("Add a new tag."),
        )
        .arg(
            Arg::new("remove")
                .short('r')
                .long("remove")
                .help("Remove an existing tag."),
        )
        .arg(
            Arg::new("update")
                .short('u')
                .long("update")
                .help("Update an existing tag."),
        )
        .arg(
            Arg::new("list")
                .short('l')
                .long("list")
                .global(true)
                .help("List all global tags or subtags of specified tag."),
        )
        .groups(&[
            ArgGroup::new("cmd-conflict")
                .args(&["add", "remove", "update", "list"])
                .multiple(false)
                .conflicts_with("cmd-req")
                .required(true),
            ArgGroup::new("cmd-req")
                .args(&["print", "copy", "silent-copy", "app"])
                .multiple(true),
        ])
        .subcommands(tags.iter().map(command_from_tag))
}
