use std::{env, fmt::Display, fs, path::PathBuf};

use clap::{Parser, Subcommand};
use serde_derive::Deserialize;
use setuprs::{copy_dir_all, search_file_create_folder_if_not_found};
use uuid::Uuid;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "TOML FILE")]
    config: Option<PathBuf>,

    /// Show the current configuration
    #[arg(long)]
    current_config: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create snapshot
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

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n----------------------\nCONFIG\n----------------------\nConfig file path: \"{}\"\nSnapshots path: \"{}\"\nDebug mode: \"{}\"\n----------------------",
            self.config_file_path, self.snapshots_path, self.debug_mode
        )
    }
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
        println!("{data}");
    }

    match &cli.command {
        Some(Commands::Snapshot { dir }) => {
            let contents = fs::read_to_string(&config_path).unwrap();
            let data: Config = toml::from_str(&contents).unwrap();
            copy_dir_all(dir, data.snapshots_path, &Uuid::new_v4().to_string()).unwrap();
        }

        None => {}
    }

    // Continued program logic goes here...
}
