# setuprs

`setuprs` is a powerful command-line interface (CLI) and text user interface
(TUI) application designed to simplify and accelerate the process of creating
snapshots of projects. By leveraging `clap.rs` for CLI functionality and
`ratatui.rs` for TUI features, `setuprs` allows users to easily "clone"
scaffolds of snapshots, streamlining project setup and management.

## Features

- **Easy Snapshot Creation**: Quickly generate snapshots of your projects.
- **Effortless Cloning**: Seamlessly clone the scaffolds of your project snapshots.
- **Intuitive CLI**: Simple and efficient command-line operations using `clap.rs`.
- **Interactive TUI**: User-friendly text interface powered by `ratatui.rs`. (in beta)

## Installation

It is not done yet, not possible to install

if you want to test the beta of the beta just clone and run

## Usage

### CLI Commands
```sh
Usage: setuprs [OPTIONS] [COMMAND]

Commands:
  snapshot  Snapshot commands
  config    Configuration options
  init      Prepare folder to create a snapshot
  help      Print this message or the help of the given subcommand(s)

Options:
  -c, --config <TOML FILE>  Sets a custom config file
  -h, --help                Print help
  -V, --version             Print version

# to initialize the folder that will be a snapshot
setuprs snapshot init

# to create a snapshot of your current dir
setuprs snapshot create

# to clone a snapshot of your current dir
setuprs snapshot clone <snapshot_name_tag> -d <path_to_clone>
```
