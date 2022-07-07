use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs};

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::tag::Tag;

pub type Tags = HashMap<String, Tag>;

/// Returns the path to the data file.
///
/// Errors if unable to retrieve the home directory path (and
/// `$OPENTAG_DATA` is not set).
pub fn get_data_path() -> Result<PathBuf> {
    env::var("OPENTAG_DATA").map_or_else(
        |_| {
            dirs_next::data_dir()
                .map(|d| d.join("opentag/tags.toml"))
                .ok_or_else(|| "unable to retrieve data directory path".into())
        },
        |p| Ok(PathBuf::from(p)),
    )
}

/// Returns the serialized tags data present at the given path.
pub fn get_data<P: AsRef<Path>>(path: P) -> Result<Tags> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .map_err(|e| format!("data file error at path `{}`: {}", path.display(), e))?;

    toml::from_str(&contents)
        .map_err(|e| format!("toml data error at path `{}`: {}", path.display(), e).into())
}

/// Writes the data at the given path, creating the file if it does not exist.
pub fn write_data<P: AsRef<Path>>(data: &Tags, path: P) -> Result<()> {
    Ok(fs::write(path, toml::to_string(data)?)?)
}

/// Recursively creates the data directory and all of its parents if they are missing.
///
/// `path` must be the path to the data FILE.
pub fn create_data_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();

    if !path.exists() {
        fs::create_dir_all(
            &path
                .parent()
                .ok_or_else(|| format!("invalid data path `{}`", path.display()))?,
        )?;
    }

    Ok(())
}
