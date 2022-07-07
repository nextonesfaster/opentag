mod app;
mod commands;
mod error;
mod tag;

use error::{exit, Result};
use tag::Tag;

fn run_app() -> Result<()> {
    let path = tag::get_tags_path()?;
    if !path.exists() {
        tag::create_tags_file(&path)?;
    }
    let mut tags = tag::get_tags(&path)?;
    let tags_clone = tags.clone();

    let mut app = app::create_tags_app(&tags_clone);
    let matches = app.get_matches_mut();

    if let Some((name, sub_matches)) = matches.subcommand() {
        if matches.contains_id("cmd-conflict") && !matches.contains_id("list") {
            return Err("this argument cannot be used with a tag".into());
        }

        if let Some(tag) = tag::find_tag(&tags, name, sub_matches) {
            commands::run_tag(tag, &matches)?;
        } else {
            return Err("no tag found".into());
        }
    } else if matches.contains_id("list") {
        if app.has_subcommands() {
            app = app.help_template("TAGS\n{subcommands}");
            for subcmd in app.get_subcommands_mut() {
                *subcmd = subcmd.clone().hide(false);
            }

            app.print_help()?;
        } else {
            println!("No tags!");
        }
    } else {
        let action = if matches.contains_id("add") {
            commands::add(&mut tags)?;
            "Added"
        } else if matches.contains_id("remove") {
            commands::remove(&mut tags)?;
            "Removed"
        } else if matches.contains_id("update") {
            commands::update(&mut tags)?;
            "Updated"
        } else {
            return Err("invalid invocation".into());
        };

        tag::write_tags(tags, &path)?;
        println!("\n{} tag.", action);
    }

    Ok(())
}

fn main() {
    run_app().unwrap_or_else(|e| exit(e, 1));
}
