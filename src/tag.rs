use std::path::{Path, PathBuf};
use std::{env, fs};

use clap::{ArgMatches, Command};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::Result;

/// Represents a tag.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Tag {
    /// The names of the tag.
    #[serde(
        alias = "name",
        deserialize_with = "deserialize_one_or_more",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub names: Vec<String>,
    /// The path to open, if any.
    #[serde(alias = "url", alias = "link", skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Short info about the tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    /// Default application to open the tag with.
    #[serde(
        alias = "default_app",
        alias = "default_application",
        skip_serializing_if = "Option::is_none"
    )]
    pub app: Option<String>,
    /// Subtags associated with the tag.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "skip_no_names"
    )]
    pub subtags: Vec<Tag>,
}

/// A collection of tags.
pub type Tags = Vec<Tag>;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
struct TagsSerde(#[serde(serialize_with = "skip_no_names")] Tags);

/// Returns the path to the tags file.
///
/// Errors if unable to retrieve the home directory path (and
/// `$OPENTAG_DATA` is not set).
pub fn get_tags_path() -> Result<PathBuf> {
    env::var("OPENTAG_DATA").map_or_else(
        |_| {
            dirs_next::data_dir()
                .map(|d| d.join("opentag/tags.json"))
                .ok_or_else(|| "unable to retrieve data directory path".into())
        },
        |p| Ok(PathBuf::from(p)),
    )
}

/// Returns the serialized tags present at the given path.
pub fn get_tags<P: AsRef<Path>>(path: P) -> Result<Tags> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .map_err(|e| format!("tags file error at path `{}`: {}", path.display(), e))?;

    serde_json::from_str::<TagsSerde>(&contents)
        .map(|t| t.0)
        .map_err(|e| format!("json error at path `{}`: {}", path.display(), e).into())
}

/// Writes the tags at the given path, creating the file if it does not exist.
pub fn write_tags<P: AsRef<Path>>(tags: Tags, path: P) -> Result<()> {
    Ok(fs::write(
        path,
        serde_json::to_string_pretty(&TagsSerde(tags))?,
    )?)
}

/// Recursively creates the tags file and all of its parent directories
/// if they are missing.
///
/// `path` must be the path to the tags FILE.
pub fn create_tags_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    };

    fs::write(path, "[]")?;

    Ok(())
}

/// Creates a `clap` subcommand for the given tag.
pub fn command_from_tag(tag: &Tag) -> Command {
    let mut cmd = Command::new(tag.names.first().expect("expected at least one name"))
        .disable_help_subcommand(true);

    if let Some(ref long_about) = tag.about {
        cmd = cmd.about(long_about.lines().next());
        cmd = cmd.long_about(long_about.as_str());
    }

    for alias in tag.names.iter().skip(1) {
        cmd = cmd.visible_alias(alias.as_str());
    }

    cmd.subcommands(tag.subtags.iter().map(command_from_tag))
}

/// Find the tag matching the command invocation.
pub fn find_tag<'a>(tags: &'a Tags, cmd: &str, matches: &ArgMatches) -> Option<&'a Tag> {
    for tag in tags {
        if tag.names.contains(&cmd.to_string()) {
            if let Some((subcmd, sub_matches)) = matches.subcommand() {
                return find_tag(&tag.subtags, subcmd, sub_matches);
            } else {
                return Some(tag);
            }
        }
    }

    None
}

/// Deserializes a string or a list of strings into a `Vec<String>`.
///
/// Returns an error if an empty list is provided.
fn deserialize_one_or_more<'de, D, T>(deserializer: D) -> std::result::Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Helper<T> {
        One(T),
        Many(Vec<T>),
    }

    Ok(match Helper::deserialize(deserializer)? {
        Helper::One(s) => vec![s],
        Helper::Many(v) => {
            if v.is_empty() {
                return Err(serde::de::Error::custom(
                    "expected at least one item, found empty array",
                ));
            } else {
                v
            }
        },
    })
}

/// Skips serializing tags with no names.
fn skip_no_names<S>(tags: &[Tag], serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let len = tags.iter().filter(|t| !t.names.is_empty()).count();
    let mut seq = serializer.serialize_seq(Some(len))?;
    for tag in tags {
        if !tag.names.is_empty() {
            seq.serialize_element(tag)?;
        }
    }
    seq.end()
}
