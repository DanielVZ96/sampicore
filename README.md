# 🐶 Sampicore

Take a screenshot, get a shareable URL

# Installation

I've only installed it on PopOS (Ubuntu/Debian based), and I needed these deps:

```sh
sudo apt update && sudo apt install libxcb-randr0-dev build-essential libssl-dev libssl-dev pkg-config libxcb1-dev libxcb-shm0-dev
```

Currently it's only published in cargo:

```sh
$ cargo install sampicore  # will download and build sampic
```

# Configuration

It consists on a single `sampic.toml` file with the following contents:

```toml
$ cat ~/.config/sampic/sampic.toml
api_key = 'S3_API_KEY'
api_secret_key = 'S3_SECRET_API_KEY'
region = 'S3_REGION_'
endpoint = 'S3_ENDPOINT'
bucket = 'sampic-store'
local_path = '/tmp'
sampic_endpoint = 'https://sampic.xyz/upload'
```

Configuration will be saved locally depending on your OS in the following directories:

(According to the [directories](https://docs.rs/directories/0.10.0/src/directories/lib.rs.html#10) rust package)

> - the [XDG base directory](https://standards.freedesktop.org/basedir-spec/basedir-spec-latest.html) and the [XDG user directory](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/) specifications on Linux,
> - the [Known Folder](<https://msdn.microsoft.com/en-us/library/windows/desktop/bb776911(v=vs.85).aspx>) system on Windows, and
> - the [Standard Directories](https://developer.apple.com/library/content/documentation/FileManagement/Conceptual/FileSystemProgrammingGuide/FileSystemOverview/FileSystemOverview.html#//apple_ref/doc/uid/TP40010672-CH2-SW6) on macOS.

# Usage

```text
$ sampic
sampic 0.1.0
Takes pictures and generates links

USAGE:
    sampic <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    config    Manage sampic configuration.
    help      Prints this message or the help of the given subcommand(s)
    local     Takes a screenshot, saves it locally and returns it's path.
    s3        Takes a screenshot, saves it in s3 and returns it's link.
    server    Runs a sampic server.
    upload    Takes a screenshot, sends it to sampic and returns it's link.
```

## upload

The easiest way to use sampic. It takes a screenshot, sends it to my own sampic server, and copies it's URL to your clipboard.

```text
$ sampic upload -h
sampic-upload
Takes a screenshot, sends it to sampic and returns it's link.

USAGE:
    sampic upload

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
```

## local

Similar to upload, but instead saves the screenshot to a local path and copies that to your clipboard.

```text
$ sampic local -h
sampic-local
Takes a screenshot, saves it locally and returns it's path.

USAGE:
    sampic local

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

```

## s3

If you have an s3-compatible bucket available, you can use this subcommand to send your screenshots there. You'll have to configure it in the sampic.toml file.

```text
$ sampic s3 -h
sampic-s3
Takes a screenshot, saves it in s3 and returns it's link.

USAGE:
    sampic s3

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
```

## config (may leave a mess in your config file)

CLI interface to change configurations. Generally works ok, but it sometimes messes with my sampic.toml.

```text
$ sampic config -h
sampic-config
Manage sampic configuration.

USAGE:
    sampic config <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help    Prints this message or the help of the given subcommand(s)
    list    List current sampic configuration values.
    set     Set sampic configuration.

```

# Future

There are some things I'd like to do in order to make sampic feature complete for my use-case:

- [ ] Crossplatform region selection. Could be done in linux with the [hacksaw](https://crates.io/crates/hacksaw) crate.
- [ ] Make sure it's secure.
- [ ] Rate limit everything with the option to register and maybe even pay to relax rate limits.
- [ ] Separate code with [feature flags](https://doc.rust-lang.org/cargo/reference/features.html).
- [ ] Tests.
- [ ] A logo.
- [ ] A homepage.

I'd love to do them all, but I'm currently working full-time, so I just work sporadically on sampic when I can during my free time.
