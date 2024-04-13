use std::{
    env,
    fmt::Display,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use serde_derive::Deserialize;
use uuid::Uuid;

pub mod cli;

#[derive(PartialEq, Deserialize, Debug)]
struct Config {
    pub config_file_path: String,
    pub debug_mode: String,
    pub snapshots_path: String,
}

impl Default for Config {
    fn default() -> Self {
        let home = env::var("HOME").unwrap();

        Self {
            config_file_path: format!("{home}/.config/setuprs/setuprs.toml"),
            debug_mode: "error".to_string(),
            snapshots_path: format!("{home}/.config/setuprs/snapshots/"),
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n----------------------\nCONFIG\n----------------------\nConfig file path: {}\nSnapshots path: {}\nDebug mode: {}\n----------------------",
            self.config_file_path, self.snapshots_path, self.debug_mode
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ParseConfigError;

impl FromStr for Config {
    type Err = ParseConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut config_file_path = None;
        let mut snapshot_path = None;
        let mut debug_mode = None;

        for line in s.lines() {
            let parts: Vec<&str> = line.split(':').map(|s| s.trim()).collect();

            if parts.len() == 2 {
                let (key, value) = (parts[0], parts[1]);

                match key {
                    "Config file path" => config_file_path = Some(value.to_string()),
                    "Snapshots path" => snapshot_path = Some(value.to_string()),
                    "Debug mode" => debug_mode = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        Ok(Config {
            config_file_path: config_file_path.unwrap(),
            snapshots_path: snapshot_path.unwrap(),
            debug_mode: debug_mode.unwrap(),
        })

        // if let (Some(config_file_path), Some(snapshots_path), Some(debug_mode)) =
        //     (config_file_path, snapshot_path, debug_mode)
        // {
        //     Ok(Config {
        //         config_file_path,
        //         snapshots_path,
        //         debug_mode,
        //     })
        // } else {
        //     Err(ParseConfigError)
        // }
    }
}

fn search_file_create_config_folder_if_not_found(
    folder_path_and_file: &str,
    Config {
        snapshots_path,
        debug_mode,
        config_file_path,
    }: &Config,
) -> Result<PathBuf, std::io::Error> {
    let file_path = Path::new(folder_path_and_file);

    // Extract the parent directory path
    let parent_dir = file_path
        .parent()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid path"))?;

    if !parent_dir.exists() {
        fs::create_dir_all(parent_dir)?;
    }

    if !file_path.exists() {
        let mut file = fs::File::create(file_path)?;
        file.write_all(
            format!(
            "config_file_path = '{config_file_path}'\ndebug_mode = '{debug_mode}'\nsnapshots_path = '{snapshots_path}'"
        )
            .as_bytes(),
        )?;
    }

    Ok(file_path.to_path_buf())
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>, id: String) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;

        if entry.file_name() == ".git" || entry.file_name() == "snapshots" || *entry.file_name() == *id {
            continue;
        }

        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()), id.clone())?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
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
        Self { folder, file }
    }
}

impl Drop for Noisy {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(format!("./{}", self.folder));
    }
}

#[test]
fn should_create_folder_and_file() {
    let Noisy { folder, file } = &Noisy::new();

    let file: String = fs::read_to_string(format!("./{folder}/{file}")).unwrap();
    assert_eq!(
        file,
        "config_file_path = '.'\ndebug_mode = 'error'\nsnapshots_path = '.'"
    );
}

#[test]
fn should_copy_folder_recurcivilly() {
    let Noisy { folder, file } = &Noisy::new();
    copy_dir_all(folder, "./test_folder_copy", "test_id".to_string()).unwrap();

    let file: String = fs::read_to_string(format!("./test_folder_copy/{file}")).unwrap();
    assert_eq!(
        file,
        "config_file_path = '.'\ndebug_mode = 'error'\nsnapshots_path = '.'"
    );

    let _ = fs::remove_dir_all("./test_folder_copy");
}
