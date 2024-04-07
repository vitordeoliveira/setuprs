use std::{
    env, fs,
    io::{Error, Write},
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use serde_derive::Deserialize;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "TOML FILE")]
    config: Option<PathBuf>,

    /// Show the current configuration
    #[arg(long)]
    current_config: bool,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Snapshot {
        #[arg(short, long)]
        dir: String,
    },
}

#[derive(Deserialize, Debug)]
struct Config {
    config_file_path: String,
    debug_mode: String,
    snapshots_path: String,
}

fn search_file_create_folder_if_not_found(
    folder_path_and_file: &str,
) -> Result<PathBuf, std::io::Error> {
    let file_path = Path::new(folder_path_and_file);

    // Extract the parent directory path
    let parent_dir = file_path
        .parent()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid path"))?;

    // Create the parent directory if it doesn't exist
    if !parent_dir.exists() {
        fs::create_dir_all(parent_dir)?;
    }

    // Create the file if it doesn't exist
    if !file_path.exists() {
        let mut file = fs::File::create(file_path)?;
        file.write_all(b"config_file_path = '.'\ndebug_mode = 'error'\nsnapshots_path = '.'")?;
    }

    // Return the full path of the file
    Ok(file_path.to_path_buf())
}

fn main() {
    let cli = Cli::parse();

    let pwd = env::var("PWD").unwrap();

    let config_path: PathBuf = match cli.config {
        Some(v) => v,
        _ => PathBuf::from(format!("{pwd}/config/.setuprs.toml")),
    };

    match search_file_create_folder_if_not_found(
        &config_path.clone().into_os_string().into_string().unwrap(),
    ) {
        Ok(path) => {
            println!("File '{:?}'", path);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }

    if cli.current_config {
        let contents = fs::read_to_string(&config_path).unwrap();
        let data: Config = toml::from_str(&contents).unwrap();
        println!("{data:?}");
    }

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    match cli.debug {
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Snapshot { dir }) => {
            println!("Dir: {dir}");
        }

        None => {}
    }

    // Continued program logic goes here...
}
