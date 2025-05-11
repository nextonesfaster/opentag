mod app;
mod commands;
mod error;
mod parser;
mod tag;

use commands::MatchFlags;
use error::{Error, Result, exit};
use tag::Tag;

fn run_app() -> Result<()> {
    let path = tag::get_tags_path()?;
    if !path.exists() {
        tag::create_tags_file(&path)?;
    }
    let mut tags = tag::get_tags(&path)?;

    let mut app = app::create_tags_app(&tags);
    let mut matches = app.get_matches_mut();

    if let Some((name, sub_matches)) = matches.remove_subcommand() {
        if commands::DEFAULT_SUBCOMMAND_NAMES.contains(&name.as_str()) {
            commands::run_global_default_command(&name, sub_matches, tags, &path)?;
        } else if let Some((tag, ssm, opt_cmd)) =
            tag::find_matching_tag(&mut tags, &name, sub_matches)
        {
            if let Some(cmd) = opt_cmd {
                // this means we hit a nested default command
                let action = commands::run_nested_default_command(tag, &cmd, ssm)?;
                tag::validate_and_write_tags(tags, &path)?;
                println!("{action} tag.");
            } else {
                commands::run_tag(tag, MatchFlags::from_matches([matches, ssm]))?;
            }
        } else {
            return Err(Error::NoTagFound.into());
        }
    } else if matches.get_flag("list") {
        if app.has_subcommands() {
            commands::list_tags(app)?;
        } else {
            println!("No tags!");
        }
    }

    Ok(())
}

fn main() {
    run_app().unwrap_or_else(|e| exit(e, 1));
}
