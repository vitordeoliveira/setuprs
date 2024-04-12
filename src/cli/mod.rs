use std::{fs, path::PathBuf};

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

// TODO: .setuprsignore.toml
// TODO: snapshots metadata -> add tags (names) to the snapshots
impl Cli {
    pub fn execute() {
        let cli = Cli::parse();

        let default_config = Config::default();

        let config_path: PathBuf = match &cli.config {
            Some(v) => v.clone(),
            _ => PathBuf::from(&default_config.config_file_path),
        };

        match search_file_create_folder_if_not_found(
            &config_path.clone().into_os_string().into_string().unwrap(),
            &default_config,
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
            let data = toml::from_str::<Config>(&contents).unwrap();
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
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use assert_cmd::Command;

    use crate::Noisy;

    use super::*;

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

            let config = Config {
                config_file_path: ".".to_string(),
                debug_mode: "error".to_string(),
                snapshots_path: ".".to_string(),
            };

            search_file_create_folder_if_not_found(format!("./{folder}/{file}").as_str(), &config)
                .unwrap();
            Self { folder, file }
        }
    }

    impl Drop for Noisy {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(format!("./{}", self.folder));
        }
    }

    #[test]
    fn current_config_should_return_correct_default_info() {
        let mut cmd = Command::cargo_bin("setuprs").unwrap();

        let value = cmd
            .arg("--current-config")
            .assert()
            .success()
            .get_output()
            .clone();

        let raw_stdout = String::from_utf8(value.stdout);

        assert_eq!(
            Config::from_str(raw_stdout.unwrap().as_ref()).unwrap(),
            Config::default()
        )
    }

    #[test]
    fn current_config_should_return_correct_info_after_define_new_config() {
        let Noisy { folder, file } = &Noisy::new();

        let mut cmd = Command::cargo_bin("setuprs").unwrap();
        cmd.arg("--config")
            .arg(format!("./{folder}/{file}"))
            .assert()
            .success();

        let value = cmd
            .arg("--current-config")
            .assert()
            .success()
            .get_output()
            .clone();

        let raw_stdout = String::from_utf8(value.stdout);

        assert_eq!(
            Config::from_str(raw_stdout.unwrap().as_ref()).unwrap(),
            Config {
                config_file_path: ".".to_string(),
                debug_mode: "error".to_string(),
                snapshots_path: ".".to_string()
            }
        )
    }

    // TODO: Test if snapshots is being created with success
    #[test]
    fn snapshots_created_with_success() {
        let Noisy { folder, file } = &Noisy::new();

        let mut cmd = Command::cargo_bin("setuprs").unwrap();
        cmd.arg("--config")
            .arg(format!("./{folder}/{file}"))
            .assert()
            .success();

        cmd.arg("snapshot").arg("-d").arg(".");
    }
}
