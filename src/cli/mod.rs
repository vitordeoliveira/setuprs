use std::{env, fs, path::PathBuf};

use assert_cmd::{assert::OutputAssertExt, Command};
use clap::{Parser, Subcommand};
use uuid::Uuid;

use crate::{copy_dir_all, search_file_create_folder_if_not_found, Config};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
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

impl Cli {
    pub fn execute() -> Result<(), ()> {
        let cli = Cli::parse();

        let pwd = env::var("PWD").unwrap();

        let config_path: PathBuf = match &cli.config {
            Some(v) => v.clone(),
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
        };

        Ok(())
    }
}

#[allow(dead_code)]
struct Noisy {
    folder: String,
    file: String,
}

impl Noisy {
    #[allow(dead_code)]
    fn new() -> Self {
        let (folder, file) = (
            Uuid::new_v4().to_string(),
            format!("{}.toml", Uuid::new_v4()),
        );
        search_file_create_folder_if_not_found(format!("./{folder}/{file}").as_str()).unwrap();
        Self { folder, file }
    }
}

impl Drop for Noisy {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(format!("./{}", self.folder));
    }
}

#[test]
fn should_call_config() {
    let Noisy { folder, file } = &Noisy::new();

    let mut cmd = Command::cargo_bin("setuprs").unwrap();
    cmd.arg("--config")
        .arg(format!("./{folder}/{file}"))
        .assert()
        .success();
    cmd.arg("--current-config").assert().success();
}
