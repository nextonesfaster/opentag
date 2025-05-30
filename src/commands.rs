use std::collections::HashSet;
use std::fmt::Write;
use std::path::PathBuf;

use arboard::Clipboard;
use clap::{ArgMatches, Command};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Editor, FuzzySelect, Input};

use crate::Tag;
use crate::error::{Error, Result};
use crate::tag::{self, Tags};

pub(crate) const DEFAULT_SUBCOMMAND_NAMES: [&str; 3] = ["add", "remove", "update"];

#[derive(Debug, Clone, Default)]
pub(crate) struct MatchOptions {
    print: bool,
    copy: bool,
    list: bool,
    silent_copy: bool,
    app: Option<String>,
    info: bool,
}

impl MatchOptions {
    pub(crate) fn from_matches<const N: usize>(lom: [ArgMatches; N]) -> Self {
        let mut flags = MatchOptions::default();

        for mut matches in lom {
            flags.list |= matches.get_flag("list");
            flags.print |= matches.get_flag("print");
            flags.copy |= matches.get_flag("copy");
            flags.silent_copy |= matches.get_flag("silent-copy");
            if flags.app.is_none() {
                flags.app = matches.remove_one::<String>("app");
            }
            flags.info |= matches
                .try_remove_one::<bool>("info")
                .ok()
                .flatten()
                .unwrap_or_default();
        }

        flags
    }
}

/// Runs the command for the given tag.
///
/// Returns `true` if the tag is updated.
pub(crate) fn run_tag(tag: &mut Tag, options: MatchOptions) -> Result<()> {
    if options.list {
        // TODO: This is a terrible hack. Write own implementation.
        if !tag.subtags.is_empty() {
            _list_tags(tag, "TAGS")?;
        } else if !options.info {
            println!("No tags!");
        }
        return Ok(());
    }

    if options.info {
        print_tag_info(tag)?;
        if !options.copy || tag.path.is_none() {
            return Ok(());
        }
    }

    let cow;
    let path = if let Some(ref path) = tag.path {
        if path.starts_with('~') {
            cow = shellexpand::tilde(path);
            cow.as_ref()
        } else {
            path.as_ref()
        }
    } else {
        return Err(Error::TagWithNoPath.into());
    };

    let silent = options.silent_copy || options.info;

    if options.copy || options.silent_copy {
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(path.to_string())?;
    }

    if options.print {
        println!("{}", path);
    } else if !silent {
        if let Some(app) = options.app.as_ref().or(tag.app.as_ref()) {
            open::with(path, app)
        } else {
            open::that(path)
        }
        .map_err(|e| format!("unable to open `{}`: {}", path, e))?;
    }

    Ok(())
}

pub(crate) fn run_global_default_command(
    name: &str,
    matches: ArgMatches,
    mut tags: Tags,
    path: &PathBuf,
) -> Result<()> {
    if name == "add" {
        if let Some(tag) = tag_from_add_matches(matches) {
            add_tag_inline(tag, &mut tags)?;
            tag::write_tags(tags, path)?;
        } else {
            interactive_add(&mut tags)?;
            tag::validate_and_write_tags(tags, path)?;
        }
        println!("\nAdded tag.");
    } else if name == "remove" {
        if interactive_remove(&mut tags, !matches.get_flag("no-prompt"))? {
            tag::write_tags(tags, path)?;
            println!("\nRemoved tag.");
        }
    } else if name == "update" && interactive_update(&mut tags)? {
        tag::validate_and_write_tags(tags, path)?;
        println!("\nUpdated tag.");
    }

    Ok(())
}

fn _list_tags(tag: &Tag, label: &str) -> Result<()> {
    let app =
        Command::new("list-subcommands").subcommands(tag.subtags.iter().map(tag::command_from_tag));
    list_tags_from_app(app, label)?;
    Ok(())
}

pub(crate) fn list_tags_from_app(mut app: Command, label: &str) -> Result<()> {
    app = app
        .help_template(format!("{label}\n{{subcommands}}"))
        .disable_help_subcommand(true);
    for subcmd in app.get_subcommands_mut() {
        *subcmd = subcmd
            .clone()
            .hide(DEFAULT_SUBCOMMAND_NAMES.contains(&subcmd.get_name())); // hide default subcommands
    }

    app.print_help()?;
    Ok(())
}

pub(crate) fn run_nested_default_command(
    tag: &mut Tag,
    command: &str,
    matches: ArgMatches,
) -> Result<&'static str> {
    match command {
        "add" => {
            let new_tag = tag_from_add_matches(matches).ok_or("tag name cannot be empty")?;
            add_tag_inline(new_tag, &mut tag.subtags)?;
            Ok("Added")
        },
        "remove" => {
            if remove_tag_inline(tag, matches)? {
                Ok("Removed")
            } else {
                Ok("Did not remove")
            }
        },
        "update" => {
            update_tag_inline(tag, matches)?;
            Ok("Updated")
        },
        _ => Err(Error::UnexpectedCommand(command.to_string()).into()),
    }
}

pub fn add_tag_inline(tag: Tag, tags: &mut Tags) -> Result<()> {
    if let Some(name) = check_if_names_are_used(&tag.names, tags) {
        return Err(Error::NameInUse(name.clone()).into());
    }

    tags.push(tag);

    Ok(())
}

fn remove_tag_inline(tag: &mut Tag, matches: ArgMatches) -> Result<bool> {
    if !matches.get_flag("no-prompt") && !remove_confirmation(&tag.names[0])? {
        return Ok(false);
    }

    // tags with no names are not written to the file
    tag.names.clear();

    Ok(true)
}

/// Updates a tag with new attributes in the matches.
///
/// This does not check if the attributes are valid or if the tag names remain unique.
fn update_tag_inline(tag: &mut Tag, mut matches: ArgMatches) -> Result<()> {
    if let Some(name) = matches.remove_one::<String>("name") {
        tag.names[0] = name;
    }

    let clear_aliases = matches.contains_id("alias");
    let aliases = matches
        .remove_one::<Vec<String>>("alias")
        .unwrap_or_default();

    if !aliases.is_empty() {
        tag.names.splice(1.., aliases);
    } else if clear_aliases {
        tag.names.truncate(1);
    }

    let mut update_if_present = |attrib, field: &mut _| {
        let is_present = matches.contains_id(attrib);
        if let Some(new) = matches.remove_one::<String>(attrib) {
            *field = Some(new);
        } else if is_present {
            *field = None;
        }
    };

    update_if_present("path", &mut tag.path);
    update_if_present("about", &mut tag.about);
    update_if_present("app", &mut tag.app);

    Ok(())
}

/// Prompts user to recursively select a tag.
fn select_tag<'a>(
    tags: &'a mut Tags,
    prompt: &str,
    rec_prompt: &str,
) -> Result<Option<&'a mut Tag>> {
    if let Some(i) = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(
            &tags
                .iter()
                .map(|t| t.names.first().expect("tag has no name"))
                .collect::<Vec<_>>(),
        )
        .interact_opt()?
    {
        let tag_ptr = tags.get_mut(i).expect("expected index in bounds") as *mut Tag;
        // SAFETY: `tag_ptr` is not mutated in this function and is valid
        let tag = unsafe { &mut *tag_ptr };
        if !tag.subtags.is_empty() {
            if let Some(t) = select_tag(&mut tag.subtags, rec_prompt, rec_prompt)? {
                return Ok(Some(t));
            }
        }
        // SAFETY: `tag_ptr` is not mutated in this function and is valid
        return Ok(Some(unsafe { &mut *tag_ptr }));
    }

    Ok(None)
}

/// Runs the add command.
fn interactive_add(tags: &mut Tags) -> Result<()> {
    let names: Vec<_> = Input::<String>::new()
        .with_prompt("Enter tag name and aliases (comma-separated; at least one)")
        .interact_text()?
        .split_terminator(',')
        .map(|s| s.trim().to_string())
        .collect();

    let subtags = if let Some(t) = select_tag(
        tags,
        "Select the parent tag (press `esc` for no parent)",
        "Select a subtag of the parent (press `esc` to select the parent)",
    )? {
        &mut t.subtags
    } else {
        tags
    };

    if let Some(name) = check_if_names_are_used(&names, subtags) {
        return Err(Error::NameInUse(name.clone()).into());
    }

    let get_optional = |prompt| -> Result<Option<String>> {
        let opt: String = Input::new()
            .with_prompt(prompt)
            .allow_empty(true)
            .interact_text()?;

        Ok(if opt.is_empty() { None } else { Some(opt) })
    };

    let path = get_optional("Enter path or url, press enter to skip")?;
    let about = get_optional("Enter info about the tag, press enter to skip")?;
    let default_application =
        get_optional("Enter name of default app to open the tag, press enter to skip")?;

    subtags.push(Tag {
        names,
        path,
        about,
        app: default_application,
        ..Default::default()
    });

    Ok(())
}

/// Runs the remove command.
fn interactive_remove(tags: &mut Tags, prompt: bool) -> Result<bool> {
    let Some(tag) = select_tag(
        tags,
        "Select the parent tag (press `esc` to quit)",
        "Select a subtag of the parent (press `esc` to select the parent)",
    )?
    else {
        return Ok(false);
    };

    if prompt && !remove_confirmation(&tag.names[0])? {
        println!("\nDid not remove tag.");
        return Ok(false);
    }

    // we take advantage of our serialization mechanism: tags with no names
    // are not written to the file.
    tag.names.clear();

    Ok(true)
}

/// Runs the update command.
fn interactive_update(tags: &mut Tags) -> Result<bool> {
    let Some(tag) = select_tag(
        tags,
        "Select the parent tag (press `esc` to quit)",
        "Select a subtag of the parent (press `esc` to select the parent)",
    )?
    else {
        return Ok(false);
    };

    let filter_text = |text: String| {
        text.lines()
            .filter(|l| {
                let trimmed = l.trim();
                !trimmed.starts_with('#') && !trimmed.is_empty()
            })
            .collect::<String>()
    };

    let ignored_str = "Lines starting with '#' will be ignored.";

    let names_msg = format!(
        "{}\n# Please enter/edit comma-separated list of names above.\n# {ignored_str}",
        tag.names.join(", ")
    );
    if let Some(names) = Editor::new().edit(&names_msg)? {
        let names = filter_text(names)
            .split_terminator(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>();
        if names.is_empty() {
            return Err(Error::MissingName.into());
        }
        tag.names = names;
    }

    let update_field = |field: &mut Option<_>, msg: &str| -> Result<()> {
        let msg = format!(
            "{}\n# {msg} {ignored_str}",
            field.as_ref().unwrap_or(&"".to_string())
        );
        if let Some(mut text) = Editor::new().edit(&msg)? {
            text = filter_text(text);
            *field = (!text.is_empty()).then_some(text);
        }

        Ok(())
    };

    update_field(&mut tag.path, "Please edit/enter the path/url above.")?;
    update_field(&mut tag.about, "Please edit/enter the description above.")?;
    update_field(&mut tag.app, "Please edit/enter the default app above.")?;

    Ok(true)
}

fn tag_from_add_matches(mut matches: ArgMatches) -> Option<Tag> {
    let name = matches.remove_one::<String>("name")?;
    let mut names = matches
        .remove_one::<Vec<String>>("alias")
        .unwrap_or_default();
    names.insert(0, name);

    Some(Tag {
        names,
        path: matches.remove_one::<String>("path"),
        about: matches.remove_one::<String>("about"),
        app: matches.remove_one::<String>("app"),
        subtags: Vec::new(),
    })
}

/// Prompts the user to confirm tag removal.
///
/// Returns `true` if the user chooses to proceed.
fn remove_confirmation(name: &str) -> Result<bool> {
    dialoguer::Confirm::new()
        .with_prompt(format!("Do you want to remove the `{}` tag?", name))
        .interact()
        .map_err(|e| e.into())
}

/// Checks if any name in `names` is already used.
///
/// Returns the first common name if any.
fn check_if_names_are_used<'a>(names: &'a [String], subtags: &[Tag]) -> Option<&'a String> {
    let mut used = HashSet::new();
    for tag in subtags {
        used.extend(&tag.names);
    }
    names.iter().find(|&name| used.contains(name))
}

/// Prints the tag info.
///
/// Does not print its subtags.
fn print_tag_info(tag: &Tag) -> Result<()> {
    let mut info_str = String::new();

    color_print::cwrite!(info_str, "<g><s>{}</></>", tag.names[0])?;

    if let Some(path) = &tag.path {
        color_print::cwriteln!(info_str, " - <u>{path}</>")?;
    } else {
        writeln!(info_str)?;
    }

    let format_label = |label| color_print::cformat!("<y><u>{}:</></>", label);

    if let Some(about) = &tag.about {
        writeln!(info_str, "\n{about}")?;
    }

    if tag.names.len() > 1 {
        writeln!(
            info_str,
            "\n{} {}",
            format_label("Aliases"),
            tag.names[1..].join(", ")
        )?;
    }

    if let Some(app) = &tag.app {
        writeln!(info_str, "\n{} {app}", format_label("App"))?;
    }

    println!("{info_str}");

    let subtags_label = format_label("Subtags");
    if tag.subtags.is_empty() {
        println!("{subtags_label} none");
    } else {
        _list_tags(tag, &subtags_label)?;
    }

    Ok(())
}
