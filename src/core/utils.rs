use std::{
    env,
    fs::{self, read_to_string, File},
    io::{self, Write},
    path::Path,
};

use uuid::Uuid;

use super::Config;

pub fn search_file_create_config_folder_if_not_found(
    folder_path_and_file: &str,
    Config {
        snapshots_path,
        debug_mode,
        config_file_path,
    }: &Config,
) -> Result<String, std::io::Error> {
    let file_path = Path::new(folder_path_and_file);

    let mut response = String::new();
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

        let val = format!("Created file: {}", file_path.display());
        response.push_str(val.as_str());
    }

    Ok(response)
}

pub fn confirm_selection() {
    let current_path = env::current_dir();
    println!("{}", current_path.unwrap().display());
}

pub fn get_all_snapshot_ids(src: impl AsRef<Path>) -> io::Result<Vec<String>> {
    let mut result: Vec<String> = vec![];
    if let Ok(entries) = fs::read_dir(src) {
        entries.for_each(|entry| {
            if let Ok(entry) = entry {
                if let Some(filename_str) = entry
                    .path()
                    .file_name()
                    .and_then(|filename| filename.to_str())
                {
                    result.push(filename_str.to_string())
                }
            }
        });
    }
    Ok(result)
}

fn ignored_files(src: impl AsRef<Path>) -> Vec<String> {
    let mut result = Vec::new();

    if let Ok(file_content) = read_to_string(src) {
        for line in file_content.lines() {
            result.push(line.to_string());
        }
    }

    result
}

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(&src)? {
        let entry = entry?;
        let ty = entry.file_type()?;

        if ignored_files(&src).contains(&entry.file_name().to_string_lossy().into_owned()) {
            continue;
        }

        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[allow(dead_code)]
struct Noisy {
    folder: String,
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
        Self { folder: uuid }
    }

    #[allow(dead_code)]
    fn add_file(self, noisy_file: NoisyFile) -> Self {
        let path = format!("./{}/{}", self.folder, noisy_file.name);
        let mut file = File::create(path).unwrap();
        file.write_all(noisy_file.content.as_bytes()).unwrap();
        self
    }
}

impl Drop for Noisy {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(format!("./{}", self.folder));
    }
}

#[test]
fn should_retrieve_a_vec_of_all_ignored_files() {
    let Noisy { folder } = &Noisy::new().add_file(NoisyFile {
        name: ".setuprsignore".to_string(),
        content: "ignored_file_0\nignored_file_1".to_string(),
    });
    let expected = vec!["ignored_file_0", "ignored_file_1"];
    let path = format!("./{folder}/.setuprsignore");
    assert_eq!(expected, ignored_files(path));
}

#[test]
fn should_create_folder_and_file() {
    let Noisy { folder } = &Noisy::new();

    let config = Config {
        config_file_path: ".".to_string(),
        debug_mode: "error".to_string(),
        snapshots_path: ".".to_string(),
    };

    let file = "file.toml".to_string();

    search_file_create_config_folder_if_not_found(format!("./{folder}/{file}").as_str(), &config)
        .unwrap();

    let file: String = fs::read_to_string(format!("./{folder}/{file}")).unwrap();
    assert_eq!(
        file,
        "config_file_path = '.'\ndebug_mode = 'error'\nsnapshots_path = '.'"
    );
}

#[test]
fn should_copy_folder_recurcivilly() {
    let Noisy { folder } = &Noisy::new();

    let config = Config {
        config_file_path: ".".to_string(),
        debug_mode: "error".to_string(),
        snapshots_path: ".".to_string(),
    };

    let file = "file.toml".to_string();

    search_file_create_config_folder_if_not_found(format!("./{folder}/{file}").as_str(), &config)
        .unwrap();

    copy_dir_all(folder, "./test_folder_copy").unwrap();

    let file: String = fs::read_to_string(format!("./test_folder_copy/{file}")).unwrap();
    assert_eq!(
        file,
        "config_file_path = '.'\ndebug_mode = 'error'\nsnapshots_path = '.'"
    );

    let _ = fs::remove_dir_all("./test_folder_copy");
}

#[test]
fn should_copy_folder_recurcivilly_ignoring_files_of_setuprsignore() {
    let Noisy { folder } = &Noisy::new().add_file(NoisyFile {
        name: ".setupignore".to_string(),
        content: "ignored_file_0\nignored_file_1".to_string(),
    });

    copy_dir_all(folder, "./test_folder_copy").unwrap();

    // let file: String = fs::read_to_string(format!("./test_folder_copy/{file}")).unwrap();
    // assert_eq!(
    //     file,
    //     "config_file_path = '.'\ndebug_mode = 'error'\nsnapshots_path = '.'"
    // );

    let _ = fs::remove_dir_all("./test_folder_copy");
}

#[test]
fn should_retrieve_id() {
    let Noisy { folder } = &Noisy::new();

    let config = Config {
        config_file_path: ".".to_string(),
        debug_mode: "error".to_string(),
        snapshots_path: ".".to_string(),
    };

    let file = "file.toml".to_string();

    search_file_create_config_folder_if_not_found(format!("./{folder}/{file}").as_str(), &config)
        .unwrap();

    let result = get_all_snapshot_ids(folder).unwrap();
    let expected = vec![format!("{file}")];
    assert_eq!(result, expected);
}
