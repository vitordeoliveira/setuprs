use std::error::Error;

use clap::Parser;
use setuprs::{
    cli::{Cli, Commands},
    core::{
        utils::{
            copy_dir_all, get_all_snapshot_ids, search_file_create_config_folder_if_not_found,
        },
        Config,
    },
    tui::app::{App, ObjList},
};
use uuid::Uuid;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

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

    if cli.current_config {
        println!("{config}");
        return Ok(());
    }

    match &cli.command {
        Some(Commands::Snapshot { dir, tag }) => {
            let id = match tag {
                Some(tag_value) => tag_value.to_string(),
                None => Uuid::new_v4().to_string(),
            };

            copy_dir_all(dir, format!("{}/{}", &config.snapshots_path, id))?;

            println!("{}", id);
        }

        Some(Commands::Init {}) => {}

        None => {
            let items_ids = get_all_snapshot_ids(&config.snapshots_path)?;
            let items = ObjList::from_array(items_ids);
            let mut app = App::new(items, config)?;
            app.run().await?;
        }
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, str::FromStr};

    use assert_cmd::Command;
    use setuprs::core::{utils::search_file_create_config_folder_if_not_found, Config};
    use uuid::Uuid;

    struct Noisy {
        configuration: Option<(String, String)>,
        cleanup: Option<Box<dyn Fn()>>,
    }

    impl Noisy {
        fn new(cleanup: Option<Box<dyn Fn()>>) -> Self {
            Self {
                configuration: None,
                cleanup,
            }
        }

        fn overwrite_cleanup(&mut self, closure: Box<dyn Fn()>) {
            self.cleanup = Some(Box::new(closure));
        }

        // fn set_configuration_folder_without_create(mut self) -> Self {
        //     if self.configuration.is_none() {
        //         let (folder, file) = (
        //             Uuid::new_v4().to_string(),
        //             format!("{}.toml", Uuid::new_v4()),
        //         );
        //
        //         self.configuration = Some((folder, file));
        //     }
        //     self
        // }

        fn set_configuration_folder(mut self) -> Self {
            if self.configuration.is_none() {
                let (folder, file) = (
                    Uuid::new_v4().to_string(),
                    format!("{}.toml", Uuid::new_v4()),
                );

                let config = Config {
                    config_file_path: ".".to_string(),
                    debug_mode: "error".to_string(),
                    snapshots_path: ".".to_string(),
                };

                search_file_create_config_folder_if_not_found(
                    format!("./{folder}/{file}").as_str(),
                    &config,
                )
                .unwrap();

                self.configuration = Some((folder, file));
            }
            self
        }
    }

    impl Drop for Noisy {
        fn drop(&mut self) {
            if let Some((folder, _)) = &self.configuration {
                let _ = fs::remove_dir_all(format!("./{}", folder));
            }

            match &self.cleanup {
                Some(f) => f(),
                None => {}
            }
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
        match &Noisy::new(None).set_configuration_folder().configuration {
            Some((folder, file)) => {
                let mut cmd = Command::cargo_bin("setuprs").unwrap();
                let value = cmd
                    .arg("--config")
                    .arg(format!("./{folder}/{file}"))
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
            None => panic!("error"),
        };
    }

    #[test]
    fn snapshots_created_with_success() {
        let mut noisy = Noisy::new(None).set_configuration_folder();

        let (folder, file) = noisy.configuration.clone().unwrap();

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
    fn snapshots_created_with_tag_success() {
        let noisy = Noisy::new(Some(Box::new(|| {
            fs::remove_dir_all("tag_name").unwrap();
        })))
        .set_configuration_folder();

        let (folder, file) = noisy.configuration.clone().unwrap();

        let mut cmd = Command::cargo_bin("setuprs").unwrap();
        let value = cmd
            .arg("--config")
            .arg(format!("./{folder}/{file}"))
            .arg("snapshot")
            .arg("-d")
            .arg(format!("./{folder}"))
            .arg("-t")
            .arg("tag_name")
            .assert()
            .success()
            .get_output()
            .clone();

        let binding = String::from_utf8(value.stdout).unwrap();
        let snapshot_file = binding.lines().next().expect("No line found").to_string();

        let read_copied_file: String =
            fs::read_to_string(format!("./{snapshot_file}/{file}")).unwrap();

        let config: Config = toml::from_str(&read_copied_file).unwrap();

        assert_eq!(snapshot_file, "tag_name");
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
