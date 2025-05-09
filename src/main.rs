mod app;
mod commands;
mod error;
mod parser;
mod tag;

use error::{Result, exit};
use tag::Tag;

fn run_app() -> Result<()> {
    let path = tag::get_tags_path()?;
    if !path.exists() {
        tag::create_tags_file(&path)?;
    }
    let mut tags = tag::get_tags(&path)?;

    let mut app = app::create_tags_app(&tags);
    let matches = app.get_matches_mut();

    if let Some((name, sub_matches)) = matches.subcommand() {
        if matches.contains_id("cmd-conflict") && !matches.get_flag("list") {
            return Err("this argument cannot be used with a tag".into());
        }

        if let Some((tag, sub_matches)) = tag::find_tag_and_sub_match(&mut tags, name, sub_matches)
        {
            let updated = commands::run_tag(tag, sub_matches)?;
            if updated {
                tag::validate_and_write_tags(tags, &path)?;
                println!("Updated tag.")
            }
        } else {
            return Err("no tag found".into());
        }
    } else if matches.get_flag("list") {
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
        let action = if matches.get_flag("add") {
            commands::add(&mut tags)?;
            "Added"
        } else if matches.get_flag("remove") {
            commands::remove(&mut tags)?;
            "Removed"
        } else if matches.get_flag("update") {
            commands::update(&mut tags)?;
            "Updated"
        } else {
            return Err("invalid invocation".into());
        };

        tag::validate_and_write_tags(tags, &path)?;
        println!("\n{} tag.", action);
    }

    Ok(())
}

fn main() {
    run_app().unwrap_or_else(|e| exit(e, 1));
}
