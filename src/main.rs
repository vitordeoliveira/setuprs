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
        search_file_create_folder_if_not_found(format!("./{folder}/{file}").as_str()).unwrap();
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

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>, id: &str) -> io::Result<()> {
    let new_dst = PathBuf::from(format!(
        "{}/{}",
        dst.as_ref().display(), // Use display() method to get path as a string
        id
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
            copy_dir_all(
                entry.path(),
                &new_dst.join(entry.file_name()),
                &Uuid::new_v4().to_string(),
            )?;
        } else {
            fs::copy(entry.path(), &new_dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[test]
fn should_copy_folder_recurcivilly() {
    let Noisy { folder, file } = &Noisy::new();
    copy_dir_all(folder, "./test_folder_copy", "test_id").unwrap();

    let file: String = fs::read_to_string(format!("./test_folder_copy/test_id/{file}")).unwrap();
    assert_eq!(
        file,
        "config_file_path = '.'\ndebug_mode = 'error'\nsnapshots_path = '.'"
    );

    let _ = fs::remove_dir_all("./test_folder_copy");
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
            copy_dir_all(dir, data.snapshots_path, &Uuid::new_v4().to_string()).unwrap();
        }

        None => {}
    }

    // Continued program logic goes here...
}
