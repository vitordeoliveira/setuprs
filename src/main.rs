use std::{
    env,
    fmt::Display,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use serde_derive::Deserialize;
use uuid::Uuid;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
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

#[derive(Deserialize, Debug)]
struct Config {
    config_file_path: String,
    debug_mode: String,
    snapshots_path: String,
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n----------------------\nCONFIG\n----------------------\nConfig file path: \"{}\"\nSnapshots path: \"{}\"\nDebug mode: \"{}\"\n----------------------",
            self.config_file_path, self.snapshots_path, self.debug_mode
        )
    }
}

fn search_file_create_folder_if_not_found(
    folder_path_and_file: &str,
) -> Result<PathBuf, std::io::Error> {
    let file_path = Path::new(folder_path_and_file);

    // Extract the parent directory path
    let parent_dir = file_path
        .parent()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid path"))?;

    // Create the parent directory if it doesn't exist
    if !parent_dir.exists() {
        fs::create_dir_all(parent_dir)?;
    }

    // Create the file if it doesn't exist
    if !file_path.exists() {
        let mut file = fs::File::create(file_path)?;
        file.write_all(b"config_file_path = '.'\ndebug_mode = 'error'\nsnapshots_path = '.'")?;
    }

    // Return the full path of the file
    Ok(file_path.to_path_buf())
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    let new_dst = PathBuf::from(format!(
        "{}/{}",
        dst.as_ref().display(), // Use display() method to get path as a string
        Uuid::new_v4()
    ));

    fs::create_dir_all(&new_dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;

        if entry.file_name() == ".git" || entry.file_name() == "snapshots" {
            println!("{:?}", entry.file_name());
            continue;
        }

        if ty.is_dir() {
            copy_dir_all(entry.path(), &new_dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), &new_dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let pwd = env::var("PWD").unwrap();

    let config_path: PathBuf = match cli.config {
        Some(v) => v,
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
            copy_dir_all(dir, data.snapshots_path).unwrap();
        }

        None => {}
    }

    // Continued program logic goes here...
}
