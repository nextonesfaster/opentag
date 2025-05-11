use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::{env, fs};

use clap::{Arg, ArgMatches, Command};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::app::{get_default_subcommands, get_global_args};
use crate::error::Result;
use crate::{Error, commands};

/// Represents a tag.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Tag {
    /// The names of the tag.
    #[serde(
        alias = "name",
        deserialize_with = "deserialize_one_or_more",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub(crate) names: Vec<String>,
    /// The path to open, if any.
    #[serde(alias = "url", alias = "link", skip_serializing_if = "Option::is_none")]
    pub(crate) path: Option<String>,
    /// Short info about the tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) about: Option<String>,
    /// Default application to open the tag with.
    #[serde(
        alias = "default_app",
        alias = "default_application",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) app: Option<String>,
    /// Subtags associated with the tag.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "skip_no_names"
    )]
    pub(crate) subtags: Vec<Tag>,
}

/// A collection of tags.
pub(crate) type Tags = Vec<Tag>;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
struct TagsSerde(#[serde(serialize_with = "skip_no_names")] Tags);

/// Returns the path to the tags file.
///
/// Errors if unable to retrieve the home directory path (and
/// `$OPENTAG_DATA` is not set).
pub(crate) fn get_tags_path() -> Result<PathBuf> {
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
pub(crate) fn get_tags<P: AsRef<Path>>(path: P) -> Result<Tags> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .map_err(|e| format!("tags file error at path `{}`: {}", path.display(), e))?;

    serde_json::from_str::<TagsSerde>(&contents)
        .map(|t| t.0)
        .map_err(|e| format!("json error at path `{}`: {}", path.display(), e).into())
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

/// Writes the tags at the given path, creating the file if it does not exist.
pub(crate) fn write_tags<P: AsRef<Path>>(tags: Tags, path: P) -> Result<()> {
    Ok(fs::write(
        path,
        serde_json::to_string_pretty(&TagsSerde(tags))?,
    )?)
}

/// Recursively creates the tags file and all of its parent directories
/// if they are missing.
///
/// `path` must be the path to the tags FILE.
pub(crate) fn create_tags_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    };

    fs::write(path, "[]")?;

    Ok(())
}

/// Checks that no two tags at the same level share a common name,
/// and that no tag name conflicts with reserved default command names.
fn validate_tags(tags: &Tags) -> Result<()> {
    fn recurse(tags: &Tags) -> Result<()> {
        let mut seen = HashSet::new();

        for tag in tags {
            for name in &tag.names {
                if seen.contains(name) {
                    return Err(Error::NameInUse(name.to_string()).into());
                }

                if commands::DEFAULT_SUBCOMMAND_NAMES.contains(&name.as_str()) {
                    return Err(Error::ReservedName(name.to_string()).into());
                }

                seen.insert(name);
            }
        }

        for tag in tags {
            recurse(&tag.subtags)?;
        }

        Ok(())
    }

    recurse(tags)
}

/// Writes the tags at the given path if they are valid.
///
/// Creates the file at path if it does not exist.
pub(crate) fn validate_and_write_tags<P: AsRef<Path>>(tags: Tags, path: P) -> Result<()> {
    validate_tags(&tags)?;
    write_tags(tags, path)
}

/// Creates a `clap` subcommand for the given tag.
pub(crate) fn command_from_tag(tag: &Tag) -> Command {
    let mut cmd = Command::new(tag.names.first().expect("expected at least one name"))
        .disable_help_subcommand(true)
        .hide(true);

    if let Some(long_about) = &tag.about {
        if let Some(about) = long_about.lines().next() {
            cmd = cmd.about(about.to_string());
        }
        cmd = cmd.long_about(long_about);
    }

    for alias in tag.names.iter().skip(1) {
        cmd = cmd.visible_alias(alias);
    }

    cmd.args(get_global_args())
        .arg(
            Arg::new("info")
                .short('i')
                .long("info")
                .action(clap::ArgAction::SetTrue)
                .conflicts_with_all(["print", "silent-copy", "list"])
                .help("Shows information about the tag"),
        )
        .subcommands(get_default_subcommands())
        .subcommands(tag.subtags.iter().map(command_from_tag))
}

/// Find the tag matching the command invocation.
///
/// Returns the matching tag and the corresponding [`ArgMatches`] for that tag.
///
/// If a default subcommand is encountered, recursion stops. In that case, the
/// returned [`ArgMatches`] corresponds to the default subcommand, not the tag itself,
/// and the third value contains the name of the default subcommand.
pub(crate) fn find_matching_tag<'a>(
    tags: &'a mut Tags,
    cmd: &str,
    mut matches: ArgMatches,
) -> Option<(&'a mut Tag, ArgMatches, Option<String>)> {
    for tag in tags {
        if tag.names.contains(&cmd.to_string()) {
            if let Some((subcmd, sub_matches)) = matches.remove_subcommand() {
                if commands::DEFAULT_SUBCOMMAND_NAMES.contains(&subcmd.as_str()) {
                    return Some((tag, sub_matches, Some(subcmd)));
                }
                return find_matching_tag(&mut tag.subtags, &subcmd, sub_matches);
            } else {
                return Some((tag, matches, None));
            }
        }
    }

    None
}
