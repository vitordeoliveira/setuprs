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

### Setting Variables

`setuprs` allows you to define variables in a `setuprs.toml` file. This makes it easy to customize your project templates. Define your variables as follows:

For example:
```toml
[[variables]]
name = "variable_name"
default = "default_value"
```

#### Using Variables in Templates
You can use these variables in your project files by enclosing the variable name in double curly braces. For instance, if you have defined a variable project_name, you can use it in your files like this:
```
{{project_name}}
```

### Filling Variable Values

When you clone a snapshot, the CLI will prompt you to enter values for these variables. If you don't provide a value, the default value specified in `setuprs.toml` will be used.

#### Example Workflow

1. Define variables in `setuprs.toml`:

    ```toml
    [[variables]]
    name = "project_name"
    default = "my_project"

    [[variables]]
    name = "author"
    default = "your_name"
    ```

2. Use these variables in your project files:

    ```rust
    // main.rs
    fn main() {
        println!("Project: {{project_name}}, Author: {{author}}");
    }
    ```

3. Clone a snapshot:

    ```sh
    setuprs snapshot clone example_snapshot -d ./new_project
    ```

4. The CLI will prompt you:

    ```
    Enter value for project_name [my_project]: 
    Enter value for author [your_name]: 
    ```

This feature ensures that you can easily and quickly customize your project scaffolds during the cloning process.
