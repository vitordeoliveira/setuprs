mod cli;
mod core;
mod error;
#[cfg(feature = "tui")]
mod tui;

use clap::Parser;
use cli::{Cli, Commands, ConfigArgs, ConfigOptions, SnapshotArgs, SnapshotOptions};
use core::{
    utils::{copy_dir_all, get_input, search_file_create_config_folder_if_not_found},
    Config, SetuprsConfig,
};
use error::*;

#[cfg(feature = "tui")]
use core::utils::get_all_snapshot_ids;

#[cfg(feature = "tui")]
use tui::app::{App, ObjList};

use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
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
        Some(Commands::Snapshot(SnapshotArgs { command })) => match command {
            SnapshotOptions::Show => {
                let snapshots_path = &config.snapshots_path;

                match fs::read_dir(snapshots_path) {
                    Ok(e) => {
                        let mut list: Vec<String> = e
                            .filter_map(|e| {
                                e.ok().and_then(|entry| {
                                    entry.file_name().to_str().map(|s| s.to_string())
                                })
                            })
                            .collect();

                        list.sort();

                        list.iter().for_each(|file| println!("{file}"));
                    }
                    Err(_) => println!("No snapshots on {}", config.snapshots_path),
                }

                return Ok(());
            }
            SnapshotOptions::Clone {
                snapshot_id,
                destination_path,
            } => {
                let snapshot_path = format!("{}{}", &config.snapshots_path, snapshot_id);
                let destination_path = destination_path.clone().unwrap_or(".".to_string());

                if !Path::new(&snapshot_path).exists() {
                    return Err(Error::SnapshotDontExist);
                };

                let setuprsconfig_path = format!("{}/setuprs.toml", snapshot_path);

                let variables = if Path::new(&setuprsconfig_path).exists() {
                    let content = fs::read_to_string(&setuprsconfig_path)?;
                    let setuprsconfig = toml::from_str::<SetuprsConfig>(&content)?;
                    setuprsconfig.variables.unwrap_or_default()
                } else {
                    vec![]
                };

                let mut answers_map: HashMap<String, String> = HashMap::new();

                for var in variables {
                    let stdio = io::stdin();
                    let input = stdio.lock();
                    let output = io::stdout();

                    let provided_value = match var.default {
                        Some(default_value) => {
                            let question = format!(
                                "Enter value for {} [default: {}]: ",
                                var.name, default_value
                            );

                            let input_value = get_input(input, output, &question);

                            match input_value.trim().is_empty() {
                                true => default_value,
                                false => input_value.trim().to_string(),
                            }
                        }

                        None => {
                            let question = format!("Enter value for {}: ", var.name);
                            get_input(input, output, &question)
                        }
                    };

                    answers_map.insert(var.name, provided_value);
                }

                let modifier = move |s: &mut String| {
                    let mut new_content = s.clone();

                    for (key, val) in answers_map.iter() {
                        new_content = new_content.replace(&format!("{{{{{key}}}}}"), val);
                    }

                    new_content
                };

                match copy_dir_all(snapshot_path, destination_path, &Some(Box::new(modifier))) {
                    Ok(v) => {
                        let path = fs::canonicalize(v)?;
                        println!("Snapshot created in: {}", path.display());
                        return Ok(());
                    }
                    Err(e) => return Err(e),
                };
            }

            SnapshotOptions::Create { project_path, name } => {
                if !Path::new(&format!("{project_path}/setuprs.toml")).exists() {
                    return Err(Error::MissingBasicInitialization);
                };

                let id = match name {
                    Some(tag_value) => tag_value.to_string(),
                    None => {
                        let setuprs_config_file_path = format!("{project_path}/setuprs.toml");
                        let content = fs::read_to_string(setuprs_config_file_path)?;
                        let setuprs_config_data = toml::from_str::<SetuprsConfig>(&content)?;

                        match setuprs_config_data.project {
                            Some(project) => project.name,
                            _ => Uuid::new_v4().to_string(),
                        }
                    }
                };

                copy_dir_all(
                    project_path,
                    format!("{}/{}", &config.snapshots_path, id),
                    &None,
                )?;

                println!("{}", id);
            }
        },

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

            let mut file_ignore_path = PathBuf::from(&current_dir);
            file_ignore_path.push(".setuprsignore");
            let mut file =
                File::create(&file_ignore_path).expect("Failed to create .setuprsignore file");
            file.write_all(b".git\nsnapshots/")
                .expect("Failed to write on .setuprsignore file");

            let stdio = io::stdin();
            let input = stdio.lock();
            let output = io::stdout();
            let project_name = get_input(input, output, "Please inform the project name:");
            let mut file_setuprs_toml = PathBuf::from(&current_dir);
            file_setuprs_toml.push("setuprs.toml");
            let mut file =
                File::create(&file_setuprs_toml).expect("Failed to create setuprs.toml file");

            let content = format!(
                "[project]
name = \"{project_name}\"\n
#[[variables]]
#name=\"variable_name\"
#default=\"variable_default_value\""
            );
            file.write_all(content.as_bytes())
                .expect("Failed to write on setuprs.toml file");
        }

        #[cfg(feature = "tui")]
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
