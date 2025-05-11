use crate::{Error, commands};

pub(crate) fn tag_name_parser(s: &str) -> Result<String, Error> {
    if s.is_empty() {
        return Err(Error::EmptyName);
    } else if s.contains(' ') {
        return Err(Error::NameWithSpaces);
    } else if s.starts_with('-') {
        return Err(Error::NameBeginsWithHyphen);
    } else if commands::DEFAULT_SUBCOMMAND_NAMES.contains(&s) {
        return Err(Error::ReservedName(s.to_string()));
    }

    Ok(s.to_string())
}

pub(crate) fn tag_aliases_parser(s: &str) -> Result<Vec<String>, Error> {
    if s.contains(' ') {
        return Err(Error::NameWithSpaces);
    } else if s.starts_with('-') {
        return Err(Error::NameBeginsWithHyphen);
    }

    let names = s.split(',').map(String::from).collect::<Vec<_>>();

    for name in &names {
        if commands::DEFAULT_SUBCOMMAND_NAMES.contains(&name.as_str()) {
            return Err(Error::ReservedName(name.to_string()));
        }
    }

    Ok(names)
}
