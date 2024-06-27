use std::{
    env,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process,
};

use clap::Parser;
use setuprs::{
    cli::{Cli, Commands, ConfigArgs, ConfigOptions},
    core::{
        utils::{
            copy_dir_all, get_all_snapshot_ids, search_file_create_config_folder_if_not_found,
        },
        Config,
    },
    error::Result,
    tui::app::{App, ObjList},
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = Config::new(&cli.config);

    match search_file_create_config_folder_if_not_found(&config.config_file_path, &config) {
        Ok(path) => {
            if !path.is_empty() {
                println!("{}", path);
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }

    match &cli.command {
        Some(Commands::Snapshot { dir, tag }) => {
            if !Path::new(&format!("{dir}/.setuprsignore")).exists() {
                eprintln!("Missing setuprs init files, please run setuprs init");
                process::exit(1);
            };

            let id = match tag {
                Some(tag_value) => tag_value.to_string(),
                None => Uuid::new_v4().to_string(),
            };

            copy_dir_all(dir, format!("{}/{}", &config.snapshots_path, id))?;

            println!("{}", id);
        }

        Some(Commands::Config(ConfigArgs { command })) => match command {
            Some(ConfigOptions::Show) => {
                println!("{config}");
            }
            _ => return Ok(()),
        },

        Some(Commands::Init { dir }) => {
            let current_dir = match dir {
                Some(dir) => dir.to_string(),
                None => {
                    env::current_dir()
                        .expect("Failed to get the current directory. Make sure the program has the necessary permissions.")
                        .display()
                        .to_string()
                }
            };
            let mut file_path = PathBuf::from(&current_dir);
            file_path.push(".setuprsignore");

            let mut file = File::create(&file_path).expect("Failed to create .setuprsignore file");

            file.write_all(b".git\nsnapshots/")
                .expect("Failed to write on .setuprsignore file");
        }

        Some(Commands::Tui {}) => {
            let items_ids = get_all_snapshot_ids(&config.snapshots_path)?;
            let items = ObjList::from_array(items_ids);
            let mut app = App::new(items, config)?;
            app.run().await?;
        }
        None => {}
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io::Write,
        path::Path,
        str::FromStr,
    };

    use assert_cmd::Command;
    use setuprs::core::{utils::search_file_create_config_folder_if_not_found, Config};
    use uuid::Uuid;

    #[allow(dead_code)]
    struct Noisy {
        folder: String,
        cleanup: Option<Box<dyn Fn()>>,
    }

    #[allow(dead_code)]
    struct NoisyFile {
        name: String,
        content: String,
    }

    impl Noisy {
        #[allow(dead_code)]
        fn new() -> Self {
            let uuid = Uuid::new_v4().to_string();
            fs::create_dir(&uuid).unwrap();
            Self {
                folder: uuid,
                cleanup: None,
            }
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
    fn on_command_create_should_ignore_files_and_folders_on_setuprsignore() {
        let mut noisy = Noisy::new()
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
            .arg("-d")
            .arg(format!("./{folder}"))
            .arg("-t")
            .arg("tag_name")
            .assert()
            .success()
            .stdout("tag_name\n");

        assert!(!Path::new(&on_folder("file1")).exists());
        assert!(!Path::new(&on_folder("folder1")).exists());
        assert!(Path::new(&on_folder("folder2")).exists());
        // assert!(!Path::new(&on_folder("folder2/file2")).exists());
    }

    #[test]
    fn on_command_init_set_default_snapshot_config_on_init() {
        let Noisy { folder, cleanup: _ } = &Noisy::new().add_config();

        let mut cmd = Command::cargo_bin("setuprs").unwrap();
        let path = format!("./{folder}/.setuprsignore");

        cmd.arg("init").arg("-d").arg(folder).assert().success();
        assert!(Path::new(&path).exists());
    }

    #[test]
    fn current_config_should_return_correct_default_info() {
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
    fn current_config_should_return_correct_info_after_define_new_config() {
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
    fn snapshots_created_with_success_without_tag() {
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
            .arg("-d")
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
    fn snapshots_should_fail_when_no_setuprsignore() {
        let noisy = &mut Noisy::new().add_config();
        let folder = noisy.folder.clone();

        let file = "file.toml".to_string();

        let mut cmd = Command::cargo_bin("setuprs").unwrap();
        cmd.arg("--config")
            .arg(format!("./{folder}/{file}"))
            .arg("snapshot")
            .arg("-d")
            .arg(format!("./{folder}"))
            .arg("-t")
            .arg("tag_name")
            .assert()
            .failure()
            .stderr("Missing setuprs init files, please run setuprs init\n");
    }

    #[test]
    fn snapshots_created_with_tag_success() {
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
            .arg("-d")
            .arg(format!("./{folder}"))
            .arg("-t")
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

    // TODO: is this test usefull also?
    // #[test]
    // fn if_folder_config_folder_not_exist_should_stdout_filepath() {
    //     let noisy = Noisy::new(None).set_configuration_folder_without_create();
    //     let (folder, file) = noisy.configuration.clone().unwrap();
    //
    //     let mut cmd = Command::cargo_bin("setuprs").unwrap();
    //
    //     let output = cmd
    //         .arg("--config")
    //         .arg(format!("./{folder}/{file}"))
    //         .assert()
    //         .success()
    //         .get_output()
    //         .clone();
    //
    //     let binding = String::from_utf8(output.stdout).unwrap(); let snapshot_file = binding.lines().next().expect("No line found").to_string();
    //
    //     let expected = format!("Created file: ./{folder}/{file}");
    //     assert_eq!(snapshot_file, expected);
    // }

    // TODO: this test idea might be useless
    // #[test]
    // fn if_folder_config_folder_exist_should_not_stdout_filepath() {
    //     let noisy = Noisy::new(None).set_configuration_folder();
    //     let (folder, file) = noisy.configuration.clone().unwrap();
    //
    //     let mut cmd = Command::cargo_bin("setuprs").unwrap();
    //     let output = cmd
    //         .arg("--config")
    //         .arg(format!("./{folder}/{file}"))
    //         .assert()
    //         .success()
    //         .get_output()
    //         .clone();
    //
    //     let binding = String::from_utf8(output.stdout).unwrap();
    //     let snapshot_file = binding.to_string();
    //
    //     assert_eq!(snapshot_file, "");
    // }
}
