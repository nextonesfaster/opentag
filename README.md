# opentag (ot)

`opentag` (binary name: `ot`) is a command-line tool that opens a tagged path or URL using the configured system program.

Tags are defined in a `json` data file. See the [Defining Tags](#defining-tags) section for more information about this file.

The tags are added to the application as "subcommands" at run-time and appear in the help text.

`opentag` is useful when you have a bunch of websites and files you regularly open. Instead of adding bookmarks to different browsers and navigating through your file system or typing the file paths and URLs by hand each time, you can simply add them as tags and instantly open them with a short command. Tags can be grouped as subtags, which can have even more subtags! You can also provide helpful descriptions for each tag.

## Defining Tags

Tags are defined in a `json` data file. You do not need to create or edit the file directly, you can use the `--add`, `--remove`, and `--update` options.

### Location

By default, the location of this file is `$DATA_DIR/opentag/tags.json` where `$DATA_DIR` is as follows:

| Platform |                `$DATA_DIR`                 |
| :------: | :----------------------------------------: |
|  Linux   |         `/home/Alice/.local/share`         |
|  macOS   | `/Users/Alice/Library/Application Support` |
| Windows  |      `C:\Users\Alice\AppData\Roaming`      |

You can override this by setting the `OPENTAG_DATA` environment variable as the path of the tags file. The environment variable takes precedence over the default location.

### Structure

The structure of the configuration file is described in the following example.

```json
[
  {
    "names": ["example", "exa"],
    "url": "https://example.com",
    "about": "Opens example.com",
    "subtags": [
      {
        "name": "readme",
        "path": "~/opentag/README.md",
        "about": "Opens the README of `opentag`"
      },
      {
        "name": "main",
        "path": "~/opentag/src/main.rs",
        "about": "Opens the `main.rs` file of `opentag`"
      }
    ]
  },
  {
    "name": "web",
    "about": "Defines web tabs. A subtag must be used.",
    "subtags": [
      {
        "names": ["github", "gh"],
        "url": "https://github.com/",
        "about": "Opens GitHub"
      }
    ]
  }
]
```

This will create two "global" tags: `example` and `web`. The `example` tag has two subtags: `readme` and `main`, and one alias: `exa`. The `web` tag has one subtag: `github`. The `github` subtag has one alias: `gh`.

Note that the `names` key has an alias (`name`) and can either be a string or a list of strings. Similarly, `url` is an alias of `path`.

## Usage

Some example commands based on the above configuration:

```sh
# VALID:

# Opens https://example.com
$ ot example
# Same as above
$ ot exa
# Opens the file at `~/opentag/README.md`
$ ot exa readme
# Opens `https://github.com`
$ ot web github
# Same as above
$ ot web gh

# Prints "https://example.com"
$ ot -p example

# Opens "https://github.com" and copies the URL to the clipboard
$ ot -c web gh

# Copies "https://github.com" to the clipboard
$ ot -C web gh

# Opens https://github.com with Firefox (if installed)
# instead of the default browser
$ ot -A firefox web gh

# Add a new tag
$ ot -a

# Remove an existing tag
$ ot -r

# Update an existing tag
$ ot -u

# INVALID:

# `exaaample` is not a valid subtag
$ ot exaaample
# `web github` is not a subtag under `example`
$ ot example web github
# `main` is not a "global" tag, it is defined under `example`
$ ot main
# `web` does not have a path or URL -- a subtag must be used
$ ot web
```

The help text generated from the above configuration is as follows:

```txt
opentag 0.0.1
Sujal Bolia <sujalbolia@gmail.com>

opentag (ot) opens a tagged path or URL using the configured system program.

USAGE:
    ot <--add|--remove|--update|--list>
    ot [OPTIONS|--list] <TAG>

OPTIONS:
    -a, --add            Add a new tag.
    -A, --app <app>      Specify the app to open the path or the URL with.
    -c, --copy           Copy the path or the URL to the system's clipboard.
    -C, --silent-copy    Copy the path or the URL to the system's clipboard without opening the path.
    -h, --help           Print help information
    -l, --list           List all global tags or subtags of specified tag.
    -p, --print          Print the path or the URL instead of opening it.
    -r, --remove         Remove an existing tag.
    -u, --update         Update an existing tag.
    -V, --version        Print version information

TAGS:
    example    Opens example.com [aliases: exv]
    web        Defines web tabs. A subtag must be used.
```

## Installation

You need [Rust][rust] to compile `opentag`.

`cargo` is usually installed with Rust. If you don't have `cargo` installed, follow [the `cargo` installation documentation][cargo].

Once you have `cargo` installed, you can simply use `cargo install` or compile from the source.

To use `cargo install`:

```sh
cargo install --git https://github.com/nextonesfaster/opentag
```

`cargo` will install `opentag` in its `bin` directory, which should already be in your `PATH`.

To compile from source:

```sh
# Clone this repository
$ git clone https://github.com/nextonesfaster/opentag.git

# cd into the cloned repository
$ cd opentag

# Compile using cargo with the release flag
$ cargo build --release
```

The executable will be at `./target/release/ot`. You can move it to your `PATH` to invoke `ot` from any directory.

## License

`opentag` is distributed under the terms of both the MIT License and the Apache License 2.0.

See the [LICENSE-MIT][mit] and [LICENSE-APACHE][apache] files for more details.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[rust]: https://www.rust-lang.org/tools/install
[cargo]: https://doc.rust-lang.org/cargo/getting-started/installation.html
[mit]: LICENSE-MIT
[apache]: LICENSE-APACHE
