use std::{fs, path::PathBuf};

use clap::Parser;
use setuprs::{
    cli::{Cli, Commands},
    copy_dir_all, search_file_create_config_folder_if_not_found, Config,
};
use uuid::Uuid;

fn main() {
    let cli = Cli::parse();

    let default_config = Config::default();

    let config_path: PathBuf = match &cli.config {
        Some(v) => v.clone(),
        _ => PathBuf::from(&default_config.config_file_path),
    };

    match search_file_create_config_folder_if_not_found(
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
            let id = Uuid::new_v4();
            copy_dir_all(
                dir,
                format!("{}/{}", data.snapshots_path, id),
                id.to_string(),
            )
            .unwrap();

            println!("{}", id);
        }

        None => {}
    };
}

#[cfg(test)]
mod tests {
    use std::{fs, str::FromStr};

    use assert_cmd::Command;
    use setuprs::{search_file_create_config_folder_if_not_found, Config};
    use uuid::Uuid;

    struct Noisy {
        configuration: Option<(String, String)>,
        cleanup: Box<dyn Fn()>,
    }

    impl Noisy {
        #[allow(dead_code)]
        fn new(closure: Box<dyn Fn()>) -> Self {
            Self {
                configuration: None,
                cleanup: closure,
            }
        }

        fn overwrite_cleanup(&mut self, closure: Box<dyn Fn()>) {
            self.cleanup = Box::new(closure);
        }

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
            println!("DROP");
            if let Some((folder, _)) = &self.configuration {
                let _ = fs::remove_dir_all(format!("./{}", folder));
            }

            (self.cleanup)();
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
        match &Noisy::new(Box::new(|| {}))
            .set_configuration_folder()
            .configuration
        {
            Some((folder, file)) => {
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
            None => panic!("error"),
        };
    }

    #[test]
    fn snapshots_created_with_success() {
        let mut noisy = Noisy::new(Box::new(|| {})).set_configuration_folder();

        let (folder, file) = noisy.configuration.clone().unwrap();

        let mut cmd = Command::cargo_bin("setuprs").unwrap();
        cmd.arg("--config")
            .arg(format!("./{folder}/{file}"))
            .assert()
            .success();

        let value = cmd
            .arg("snapshot")
            .arg("-d")
            .arg(format!("./{folder}"))
            .assert()
            .get_output()
            .clone();

        let binding = String::from_utf8(value.stdout).unwrap();
        let snapshot_file = binding
            .lines()
            .nth(1)
            .expect("No second line found")
            .to_string();

        let snapshot_file_clone = snapshot_file.clone();
        noisy.overwrite_cleanup(Box::new(move || {
            println!("DEFINED");
            fs::remove_dir_all(&snapshot_file_clone).unwrap();
        }));

        println!("READCOPY");

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
}
