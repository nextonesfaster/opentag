use std::fmt::Display;
use std::io::Write;

use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

/// Result type used throughout the crate.
pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Error type used throughout the crate.
#[derive(Debug, Clone)]
pub(crate) enum Error {
    NameInUse(String),
    ReservedName(String),
    MissingName,
    NoTagFound,
    EmptyName,
    NameWithSpaces,
    NameBeginsWithHyphen,
    TagWithNoPath,
    UnexpectedCommand(String),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NameInUse(n) => write!(f, "a tag with name `{n}` already exists"),
            Error::ReservedName(n) => write!(f, "`{n}` cannot be used as a tag name"),
            Error::MissingName => write!(f, "there must be at least one name"),
            Error::NoTagFound => write!(f, "no tag found"),
            Error::EmptyName => write!(f, "tag names cannot be empty"),
            Error::NameWithSpaces => write!(f, "tag names cannot contain spaces"),
            Error::NameBeginsWithHyphen => write!(f, "tag names cannot begin with hyphens"),
            Error::TagWithNoPath => write!(f, "tag has no path or URL"),
            Error::UnexpectedCommand(c) => write!(f, "unexpected command: {c}"),
        }
    }
}

/// Prints the error on the `stderr` and exits with the provided exit code.
///
/// "error: " is displayed before the error message. The "error" is displayed in
/// red and bold if possible.
pub(crate) fn exit<T: Display>(err: T, code: i32) -> ! {
    print_error(&err).unwrap_or_else(|_| eprintln!("error: {}", err));
    std::process::exit(code);
}

/// Prints error on the `stderr`.
///
/// "error: " is displayed before the error message. The "error" is displayed in
/// red and bold if possible.
fn print_error<T: Display>(err: &T) -> Result<()> {
    let bufwtr = BufferWriter::stderr(ColorChoice::Auto);
    let mut buffer = bufwtr.buffer();

    buffer.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;

    write!(&mut buffer, "error")?;
    buffer.reset()?;
    writeln!(&mut buffer, ": {}", err)?;

    bufwtr.print(&buffer)?;

    Ok(())
}
