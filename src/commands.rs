use arboard::Clipboard;
use clap::{ArgMatches, Command};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Editor, FuzzySelect, Input};
use itertools::Itertools;

use crate::error::Result;
use crate::tag::{command_from_tag, Tags};
use crate::Tag;

/// Runs the command for the given tag.
pub fn run_tag(tag: &Tag, matches: &ArgMatches) -> Result<()> {
    if matches.contains_id("list") {
        // TODO: This is a terrible hack. Write own implementation.
        if !tag.subtags.is_empty() {
            let mut app = Command::new("list-subcommands")
                .subcommands(tag.subtags.iter().map(command_from_tag))
                .disable_help_subcommand(true)
                .help_template("TAGS\n{subcommands}");
            app.print_help()?;
        } else {
            println!("No tags!");
        }
        return Ok(());
    }

    let mut path = if let Some(ref path) = tag.path {
        path.as_str()
    } else {
        return Err("tag has no path or url".into());
    };

    let silent_copy = matches.contains_id("silent-copy");

    if matches.contains_id("copy") || silent_copy {
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(path.to_string())?;
    }

    if matches.contains_id("print") {
        println!("{}", path);
    } else if !silent_copy {
        let cow;
        if path.starts_with('~') {
            cow = shellexpand::tilde(path);
            path = cow.as_ref();
        }

        if let Some(app) = matches.value_of("app").or(tag.app.as_deref()) {
            open::with(path, app)
        } else {
            open::that(path)
        }
        .map_err(|e| format!("unable to open `{}`: {}", path, e))?;
    }

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
pub fn add(tags: &mut Tags) -> Result<()> {
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

    for name in &names {
        if subtags.iter().flat_map(|t| &t.names).contains(name) {
            return Err(format!("a tag with name `{}` already exists", name).into());
        }
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
pub fn remove(tags: &mut Tags) -> Result<()> {
    if let Some(tag) = select_tag(
        tags,
        "Select the parent tag (press `esc` to quit)",
        "Select a subtag of the parent (press `esc` to select the parent)",
    )? {
        // we take advantage of our serialization mechanism: tags with no names
        // are not written to the file.
        tag.names.clear();
    };

    Ok(())
}

/// Runs the update command.
pub fn update(tags: &mut Tags) -> Result<()> {
    let tag = match select_tag(
        tags,
        "Select the parent tag (press `esc` to quit)",
        "Select a subtag of the parent (press `esc` to select the parent)",
    )? {
        Some(t) => t,
        None => return Ok(()),
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
            return Err("there must be at least one name".into());
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
            *field = (!text.is_empty()).then(|| text);
        }

        Ok(())
    };

    update_field(&mut tag.path, "Please edit/enter the path/url above.")?;
    update_field(&mut tag.about, "Please edit/enter the description above.")?;
    update_field(&mut tag.app, "Please edit/enter the default app above.")
}
