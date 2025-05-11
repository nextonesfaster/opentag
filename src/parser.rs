use crate::commands;

pub(crate) fn tag_name_parser(s: &str) -> Result<String, String> {
    if s.is_empty() {
        return Err("tag names cannot be empty".to_string());
    } else if s.contains(' ') {
        return Err("tag names cannot contain spaces".to_string());
    } else if s.starts_with('-') {
        return Err("tag names cannot begin with hyphens".to_string());
    } else if commands::DEFAULT_SUBCOMMAND_NAMES.contains(&s) {
        return Err(format!("`{s}` cannot be used as a tag name"));
    }

    Ok(s.to_string())
}

pub(crate) fn tag_aliases_parser(s: &str) -> Result<Vec<String>, String> {
    if s.contains(' ') {
        return Err("tag aliases cannot contain spaces".to_string());
    } else if s.starts_with('-') {
        return Err("tag aliases cannot begin with hyphens".to_string());
    }

    let names = s.split(',').map(String::from).collect::<Vec<_>>();

    for name in &names {
        if commands::DEFAULT_SUBCOMMAND_NAMES.contains(&name.as_str()) {
            return Err(format!("`{name}` cannot be used as a tag name"));
        }
    }

    Ok(names)
}
