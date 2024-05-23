use std::{env, fmt::Display, fs, path::PathBuf, str::FromStr};

use serde_derive::Deserialize;
pub mod utils;

#[derive(PartialEq, Deserialize, Debug)]
pub struct Config {
    pub config_file_path: String,
    pub debug_mode: String,
    pub snapshots_path: String,
}

impl Config {
    pub fn new(config: &Option<PathBuf>) -> Self {
        let config_path: PathBuf = match config {
            Some(v) => v.clone(),
            _ => PathBuf::from(Self::default().config_file_path),
        };

        if let Ok(contents) = fs::read_to_string(config_path) {
            if let Ok(newconf) = toml::from_str::<Config>(&contents) {
                newconf
            } else {
                Config::default()
            }
        } else {
            Config::default()
        }
    }
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
pub struct ParseConfigError;

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
