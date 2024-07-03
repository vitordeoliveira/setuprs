use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
pub struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "TOML FILE")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Snapshot commands
    Snapshot(SnapshotArgs),

    /// Configuration options
    Config(ConfigArgs),

    /// Prepare folder to create a snapshot
    Init {
        #[arg(short, long)]
        dir: Option<String>,
    },

    /// Run terminal-user-interface
    Tui {},
}
#[derive(Debug, Args)]
#[command(arg_required_else_help = true)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: Option<ConfigOptions>,
}

#[derive(Debug, Subcommand)]
pub enum ConfigOptions {
    /// Show the current configuration
    Show,
}

#[derive(Debug, Args)]
#[command(arg_required_else_help = true)]
pub struct SnapshotArgs {
    #[command(subcommand)]
    pub command: SnapshotOptions,
}

#[derive(Debug, Subcommand)]
pub enum SnapshotOptions {
    /// Create new snapshot
    #[command(arg_required_else_help = true)]
    Create {
        /// Define FROM here setuprs should create the snapshot
        project_path: String,

        /// If set will create a name for the snapshot, if not will create an unique ID
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Clone snapshot
    #[command(arg_required_else_help = true)]
    Clone {
        /// Select snapshot
        snapshot: String,

        /// Define TO here setuprs should clone the snapshot
        #[arg(short, long)]
        destination_path: Option<String>,
    },

    /// Show all snapshots_path
    Show,
}

// TODO: snapshots metadata
#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io::Write,
        path::Path,
        str::FromStr,
    };

    use assert_cmd::Command;
    use serial_test::serial;
    use uuid::Uuid;

    use crate::core::{
        utils::{copy_dir_all, search_file_create_config_folder_if_not_found},
        Config,
    };

    #[allow(dead_code)]
    struct Noisy {
        folder: String,
        cleanup: Option<Box<dyn Fn()>>,
    }

    struct NoisyFile {
        name: String,
        content: String,
    }

    impl Noisy {
        fn new() -> Self {
            let uuid = Uuid::new_v4().to_string();
            fs::create_dir(&uuid).unwrap();
            Self {
                folder: uuid,
                cleanup: None,
            }
        }

        fn folder(&self) -> String {
            self.folder.clone()
        }

        fn add_file(self, noisy_file: NoisyFile) -> Self {
            let path = format!("./{}/{}", self.folder, noisy_file.name);
            let mut file = File::create(path).unwrap();
            file.write_all(noisy_file.content.as_bytes()).unwrap();
            self
        }

        fn add_config(self) -> Self {
            let config = Config {
                config_file_path: ".".to_string(),
                debug_mode: "error".to_string(),
                snapshots_path: ".".to_string(),
            };

            let file = "file.toml".to_string();

            search_file_create_config_folder_if_not_found(
                format!("./{}/{file}", self.folder).as_str(),
                &config,
            )
            .unwrap();
            self
        }

        fn overwrite_cleanup(&mut self, closure: Box<dyn Fn()>) {
            self.cleanup = Some(Box::new(closure));
        }

        fn add_folder(self, folder_name: String) -> Self {
            let path = format!("./{}/{}", self.folder, folder_name);
            fs::create_dir(path).unwrap();
            self
        }
    }

    impl Drop for Noisy {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(format!("./{}", self.folder));

            match &self.cleanup {
                Some(f) => f(),
                None => {}
            }
        }
    }

    #[test]
    #[serial]
    fn on_snapshot_create_should_ignore_files_and_folders_on_setuprsignore() {
        let noisy = &mut Noisy::new()
            .add_config()
            .add_file(NoisyFile {
                name: ".setuprsignore".to_string(),
                content: "file1\nfolder1\nfolder2/file2".to_string(),
            })
            .add_file(NoisyFile {
                name: "file1".to_string(),
                content: "".to_string(),
            })
            .add_folder("folder1".to_string())
            .add_folder("folder2".to_string())
            .add_file(NoisyFile {
                name: "folder2/file2".to_string(),
                content: "".to_string(),
            });

        let folder = noisy.folder.clone();

        noisy.overwrite_cleanup(Box::new(move || {
            fs::remove_dir_all("tag_name").unwrap();
        }));

        let mut cmd = Command::cargo_bin("setuprs").unwrap();

        let on_folder = |file: &str| -> String { format!("./tag_name/{file}") };

        cmd.arg("--config")
            .arg(format!("./{folder}/file.toml"))
            .arg("snapshot")
            .arg("create")
            .arg(format!("./{folder}"))
            .arg("-n")
            .arg("tag_name")
            .assert()
            .success()
            .stdout("tag_name\n");

        assert!(!Path::new(&on_folder("file1")).exists());
        assert!(!Path::new(&on_folder("folder1")).exists());
        assert!(Path::new(&on_folder("folder2")).exists());
        assert!(!Path::new(&on_folder("folder2/file2")).exists());
    }

    #[test]
    fn on_snapshot_show_should_return_snapshots() {
        let noisy = &mut Noisy::new().add_config();

        let folder = noisy.folder();

        let mut cmd = Command::cargo_bin("setuprs").unwrap();

        // create a snapshots by force
        // copy_dir_all(&folder, "./snapshot_id").unwrap();

        cmd.arg("--config")
            .arg(format!("./{folder}/file.toml"))
            .arg("snapshot")
            .arg("show")
            .assert()
            .success()
            .stdout("snapshot_id");
    }

    // #[test]
    // fn on_snapshot_clone_should_copy_from_snapshot_id() {
    //     let noisy = &mut Noisy::new().add_config();
    //
    //     let folder = noisy.folder();
    //
    //     let mut cmd = Command::cargo_bin("setuprs").unwrap();
    //
    //     // create a snapshots by force
    //     copy_dir_all(&folder, "./snapshot_id").unwrap();
    //
    //     cmd.arg("--config")
    //         .arg(format!("./{folder}/file.toml"))
    //         .arg("clone")
    //         .arg("snapshot_id")
    //         .arg("-d")
    //         .arg(folder)
    //         .assert()
    //         .success();
    // }

    #[test]
    fn on_init_set_default_snapshot_config_on_init() {
        let Noisy { folder, cleanup: _ } = &Noisy::new().add_config();

        let mut cmd = Command::cargo_bin("setuprs").unwrap();
        let path = format!("./{folder}/.setuprsignore");

        cmd.arg("init").arg("-d").arg(folder).assert().success();
        assert!(Path::new(&path).exists());
    }

    #[test]
    fn on_config_current_config_should_return_correct_default_info() {
        let mut cmd = Command::cargo_bin("setuprs").unwrap();

        let value = cmd
            .arg("config")
            .arg("show")
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
    fn on_config_current_config_should_return_correct_info_after_define_new_config() {
        let Noisy { folder, cleanup: _ } = &Noisy::new().add_config();

        let mut cmd = Command::cargo_bin("setuprs").unwrap();
        let value = cmd
            .arg("--config")
            .arg(format!("./{folder}/file.toml"))
            .arg("config")
            .arg("show")
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

    #[test]
    fn on_snapshot_create_snapshots_created_with_success_without_tag() {
        let noisy = &mut Noisy::new().add_config().add_file(NoisyFile {
            name: ".setuprsignore".to_string(),
            content: "".to_string(),
        });
        let folder = &noisy.folder;
        let file = "file.toml".to_string();

        let mut cmd = Command::cargo_bin("setuprs").unwrap();
        let value = cmd
            .arg("--config")
            .arg(format!("./{folder}/{file}"))
            .arg("snapshot")
            .arg("create")
            .arg(format!("./{folder}"))
            .assert()
            .get_output()
            .clone();

        let binding = String::from_utf8(value.stdout).unwrap();
        let snapshot_file = binding
            .lines()
            .next()
            .expect("No second line found")
            .to_string();

        let snapshot_file_clone = snapshot_file.clone();

        noisy.overwrite_cleanup(Box::new(move || {
            fs::remove_dir_all(&snapshot_file_clone).unwrap();
        }));

        let read_copied_file: String =
            fs::read_to_string(format!("./{snapshot_file}/{file}")).unwrap();

        let config: Config = toml::from_str(&read_copied_file).unwrap();

        assert_eq!(
            config,
            Config {
                config_file_path: ".".to_string(),
                debug_mode: "error".to_string(),
                snapshots_path: ".".to_string()
            }
        );
    }

    #[test]
    fn on_snapshot_create_snapshots_should_fail_when_no_setuprsignore() {
        let noisy = &mut Noisy::new().add_config();
        let folder = noisy.folder.clone();

        let file = "file.toml".to_string();

        let mut cmd = Command::cargo_bin("setuprs").unwrap();
        cmd.arg("--config")
            .arg(format!("./{folder}/{file}"))
            .arg("snapshot")
            .arg("create")
            .arg(format!("./{folder}"))
            .assert()
            .failure()
            .stderr(predicates::str::contains(
                "Error: Missing setuprs init files, please run setuprs init",
            ));
    }

    #[test]
    #[serial]
    fn on_snapshot_create_snapshots_created_with_tag_success() {
        let noisy = &mut Noisy::new().add_config().add_file(NoisyFile {
            name: ".setuprsignore".to_string(),
            content: "".to_string(),
        });
        let folder = noisy.folder.clone();

        let file = "file.toml".to_string();

        noisy.overwrite_cleanup(Box::new(move || {
            fs::remove_dir_all("tag_name").unwrap();
        }));

        let mut cmd = Command::cargo_bin("setuprs").unwrap();
        cmd.arg("--config")
            .arg(format!("./{folder}/{file}"))
            .arg("snapshot")
            .arg("create")
            .arg(format!("./{folder}"))
            .arg("-n")
            .arg("tag_name")
            .assert()
            .success()
            .stdout("tag_name\n");

        let read_copied_file: String = fs::read_to_string(format!("./tag_name/{file}")).unwrap();
        let config: Config = toml::from_str(&read_copied_file).unwrap();

        assert_eq!(
            config,
            Config {
                config_file_path: ".".to_string(),
                debug_mode: "error".to_string(),
                snapshots_path: ".".to_string()
            }
        );
    }

    #[test]
    fn should_return_helper_message() {
        let mut cmd = Command::cargo_bin("setuprs").unwrap();

        cmd.assert().failure().stderr(predicates::str::contains(
            "Usage: setuprs [OPTIONS] [COMMAND]

Commands:
  snapshot  Snapshot commands
  config    Configuration options
  init      Prepare folder to create a snapshot
  tui       Run terminal-user-interface
  help      Print this message or the help of the given subcommand(s)

Options:
  -c, --config <TOML FILE>  Sets a custom config file
  -h, --help                Print help
  -V, --version             Print version",
        ));
    }
}
