use clap::Parser;
use setuprs::{
    cli::{Cli, Commands, ConfigArgs, ConfigOptions, SnapshotArgs, SnapshotOptions},
    core::{
        utils::{
            copy_dir_all, get_all_snapshot_ids, search_file_create_config_folder_if_not_found,
        },
        Config,
    },
    error::{Error, Result},
    tui::app::{App, ObjList},
};
use std::{
    env,
    fs::{self, File},
    io::Write,
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
                    Ok(e) => e
                        .filter_map(|e| e.ok()) // Filter out Err variants
                        .for_each(|entry| println!("{:?}", entry.file_name())),
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

                match copy_dir_all(snapshot_path, destination_path) {
                    Ok(v) => {
                        let path = fs::canonicalize(v)?;
                        println!("Snapshot created in: {}", path.display());
                        return Ok(());
                    }
                    Err(e) => return Err(e),
                };
            }

            SnapshotOptions::Create { project_path, name } => {
                if !Path::new(&format!("{project_path}/.setuprsignore")).exists() {
                    return Err(Error::MissingBasicInitialization);
                };

                let id = match name {
                    Some(tag_value) => tag_value.to_string(),
                    None => Uuid::new_v4().to_string(),
                };

                copy_dir_all(project_path, format!("{}/{}", &config.snapshots_path, id))?;

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
