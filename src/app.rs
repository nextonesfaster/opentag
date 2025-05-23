use clap::builder::Styles;
use clap::builder::styling::AnsiColor;
use clap::{Arg, ArgAction, ArgGroup, Command};

use crate::parser::{tag_aliases_parser, tag_name_parser};
use crate::tag::{Tags, command_from_tag};

const ABOUT: &str = "opentag (ot) opens tagged files or URLs using the default system application.

It provides a convenient way to manage frequently accessed resources—such as
websites, documents, or directories—without relying on browser bookmarks or manual
path entry. Resources are organized using user-defined tags, which can be nested
hierarchically via subtags. Each tag may also include a description for reference.

For setup usage examples and data format, refer to the project repository.

Project home page: https://github.com/nextonesfaster/opentag
";

const HELP_TEMPLATE: &str = color_print::cstr!(
    r#"{before-help}<g><s>{bin}</></> {version}
{author-with-newline}
{about-with-newline}
{usage-heading} <g>{bin-name} [OPTIONS] [COMMAND-OR-TAG]</>

{all-args}{after-help}"#
);

pub(crate) fn create_tags_app(tags: &Tags) -> Command {
    let app = clap::command!()
        .arg_required_else_help(true)
        .subcommand_negates_reqs(true)
        .disable_help_subcommand(true)
        .about(ABOUT.trim_start().lines().next())
        .long_about(ABOUT)
        .hide_possible_values(true)
        .styles(
            Styles::styled()
                .header(AnsiColor::Yellow.on_default().bold().underline())
                .usage(AnsiColor::Yellow.on_default().bold().underline())
                .literal(AnsiColor::Green.on_default())
                .placeholder(AnsiColor::Green.on_default())
                .valid(AnsiColor::Cyan.on_default()),
        )
        .args(get_global_args())
        .group(
            ArgGroup::new("cmd-req")
                .args(["print", "copy", "silent-copy", "app"])
                .multiple(true),
        )
        .subcommands(get_default_subcommands())
        .subcommands(tags.iter().map(command_from_tag));

    app.help_template(get_help_template())
}

pub(crate) fn get_global_args() -> [Arg; 5] {
    [
        Arg::new("print")
            .short('p')
            .long("print")
            .action(ArgAction::SetTrue)
            .help("Print the path or the URL instead of opening it"),
        Arg::new("app")
            .short('A')
            .long("app")
            .num_args(1)
            .value_name("APP-NAME")
            .conflicts_with_all(["print", "silent-copy"])
            .help("Specify the app to open the path or the URL with"),
        Arg::new("copy")
            .short('c')
            .long("copy")
            .action(ArgAction::SetTrue)
            .help("Copy the path or the URL to the system clipboard"),
        Arg::new("silent-copy")
            .short('C')
            .long("silent-copy")
            .action(ArgAction::SetTrue)
            .help("Copy the path or the URL to the system clipboard without opening it"),
        Arg::new("list")
            .short('l')
            .long("list")
            .conflicts_with_all(["copy", "print", "app", "silent-copy"])
            .action(ArgAction::SetTrue)
            .help("List all global tags or subtags of specified tag"),
    ]
}

pub(crate) fn get_default_subcommands() -> [Command; 3] {
    let common_args = [
        Arg::new("path")
            .visible_aliases(["link", "url"])
            .short('p')
            .visible_short_aliases(['l', 'u'])
            .long("path")
            .num_args(0..=1)
            .value_name("PATH")
            .help("Set the path/URL of the tag"),
        Arg::new("alias")
            .short('A')
            .long("alias")
            .visible_alias("aliases")
            .value_name("ALIAS(ES)")
            .value_parser(tag_aliases_parser)
            .num_args(0..=1)
            .help("Set alias(es) for the tag")
            .long_help("Set alias(es) for the tag. Multiple aliases must be comma-separated."),
        Arg::new("about")
            .long("about")
            .num_args(0..=1)
            .value_name("TEXT")
            .help("Set the about text for the tag"),
        Arg::new("app")
            .long("app")
            .num_args(0..=1)
            .value_name("APP-NAME")
            .help("Specify the app to open the path or the URL with"),
    ];

    [
        Command::new("add")
            .visible_short_flag_alias('a')
            .arg(
                Arg::new("name")
                    .value_parser(tag_name_parser)
                    .value_name("TAG-NAME")
                    .help("Set the name of the tag"),
            )
            .args(common_args.clone())
            .about("Add a new tag")
            .long_about("Add a new tag. If no name is provided, the command enters interactive mode."),
        Command::new("remove")
            .visible_short_flag_alias('r')
            .arg(
                Arg::new("no-prompt")
                    .short('N')
                    .long("no-prompt")
                    .action(ArgAction::SetTrue)
                    .help("Disable the confirmation prompt when removing a tag"),
            )
            .about("Remove an existing tag")
            .long_about("Remove an existing tag. If no tag is specified, the command enters interactive mode."),
        Command::new("update")
            .visible_short_flag_alias('u')
            .arg(
                Arg::new("name")
                    .short('n')
                    .long("name")
                    .value_name("TAG-NAME")
                    .value_parser(tag_name_parser)
                    .help("Set the name of the tag"),
            )
            .args(common_args)
            .about("Update an existing tag")
            .long_about("Update an existing tag. If no tag is specified, the command enters interactive mode."),
    ]
}

fn get_help_template() -> String {
    HELP_TEMPLATE.replace("{bin-name}", option_env!("CARGO_BIN_NAME").unwrap_or("ot"))
}
