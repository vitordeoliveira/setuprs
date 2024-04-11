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
            snapshots_path: format!("{home}/.config/setuprs/"),
        }
    }
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

pub fn search_file_create_folder_if_not_found(
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

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>, id: &str) -> io::Result<()> {
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
