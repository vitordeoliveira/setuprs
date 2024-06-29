use glob::Pattern;
use lazy_static::lazy_static;
use std::{
    env, fs,
    io::{self, Write},
    path::Path,
    sync::Mutex,
};

use super::Config;

lazy_static! {
    static ref SETUPRSIGNORE: Mutex<Option<Vec<Pattern>>> = Mutex::new(None);
}

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

fn is_ignored(path: &Path) -> bool {
    let setup = SETUPRSIGNORE.lock().unwrap();

    if let Some(ref ignore_patterns) = *setup {
        if let Some(path_str) = path.to_str() {
            println!("PATTERN: {}", path_str);
            for pattern in ignore_patterns.iter() {
                if pattern.matches(path_str) {
                    return true;
                }
            }
        }
    }
    false
}

fn load_gitignore_patterns(path: &Path) -> Vec<Pattern> {
    let path = format!("{}/.setuprsignore", path.display());
    let mut patterns = Vec::new();
    if let Ok(lines) = fs::read_to_string(path) {
        for line in lines.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                if let Ok(pattern) = Pattern::new(trimmed) {
                    patterns.push(pattern);
                }
            }
        }
    }
    patterns
}

fn get_or_set_value(new_value: Vec<Pattern>) {
    let mut setup = SETUPRSIGNORE.lock().unwrap();
    if setup.is_none() {
        *setup = Some(new_value);
    }
}

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(&src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        get_or_set_value(load_gitignore_patterns(src.as_ref()));

        if is_ignored(&entry.path()) {
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

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io::Write,
        path::Path,
    };

    use glob::Pattern;
    use serial_test::serial;
    use uuid::Uuid;

    use crate::core::{
        utils::{
            copy_dir_all, get_all_snapshot_ids, is_ignored, load_gitignore_patterns,
            search_file_create_config_folder_if_not_found,
        },
        Config,
    };

    use super::SETUPRSIGNORE;

    #[allow(dead_code)]
    struct Noisy {
        folder: String,
        cleanup: Option<Box<dyn Fn()>>,
    }

    #[allow(dead_code)]
    struct NoisyFile {
        name: String,
        content: String,
    }

    impl Noisy {
        fn new() -> Self {
            let uuid = Uuid::new_v4().to_string();
            fs::create_dir(&uuid).unwrap();
            Self {
                folder: uuid,
                cleanup: None,
            }
        }

        fn add_folder(self, folder_name: String) -> Self {
            let path = format!("./{}/{}", self.folder, folder_name);
            fs::create_dir(path).unwrap();
            self
        }

        fn overwrite_cleanup(&mut self, closure: Box<dyn Fn()>) {
            self.cleanup = Some(Box::new(closure));
        }

        fn add_file(self, noisy_file: NoisyFile) -> Self {
            let path = format!("./{}/{}", self.folder, noisy_file.name);
            let mut file = File::create(path).unwrap();
            file.write_all(noisy_file.content.as_bytes()).unwrap();
            self
        }

        fn add_file_non_method(folder: String, noisy_file: NoisyFile) {
            let path = format!("./{}/{}", folder, noisy_file.name);
            let mut file = File::create(path).unwrap();
            file.write_all(noisy_file.content.as_bytes()).unwrap();
        }
    }

    impl Drop for Noisy {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(format!("./{}", self.folder));

            match &self.cleanup {
                Some(f) => f(),
                None => {}
            }
        }
    }

    fn set_value(new_value: Vec<Pattern>) {
        println!("SET_VALUE: {:?}", new_value);
        let mut setup = SETUPRSIGNORE.lock().unwrap();
        *setup = Some(new_value);
    }

    #[test]
    #[serial]
    fn should_return_true_when_file_is_on_ignore() {
        let Noisy { folder, cleanup: _ } = &Noisy::new().add_file(NoisyFile {
            name: ".setuprsignore".to_string(),
            content: "ignored_file_0\nignored_file_1\nfolder/ignored_file_2".to_string(),
        });

        set_value(load_gitignore_patterns(Path::new(folder)));

        assert!(is_ignored(Path::new("ignored_file_0")));
        assert!(is_ignored(Path::new("ignored_file_1")));
        assert!(is_ignored(Path::new("folder/ignored_file_2")));

        assert!(!is_ignored(Path::new("file_1")));
    }

    #[test]
    #[serial]
    fn should_return_false_when_file_is_not_on_ignore() {
        let Noisy { folder, cleanup: _ } = &Noisy::new().add_file(NoisyFile {
            name: ".setuprsignore".to_string(),
            content: "ignored_file_0\nignored_file_1\nfolder/ignored_file_2".to_string(),
        });

        set_value(load_gitignore_patterns(Path::new(folder)));

        assert!(!is_ignored(Path::new("file_1")));
        assert!(is_ignored(Path::new("ignored_file_0")));
    }

    #[test]
    #[serial]
    fn should_create_folder_and_file() {
        let Noisy { folder, cleanup: _ } = &Noisy::new();

        let config = Config {
            config_file_path: ".".to_string(),
            debug_mode: "error".to_string(),
            snapshots_path: ".".to_string(),
        };

        let file = "file.toml".to_string();

        search_file_create_config_folder_if_not_found(
            format!("./{folder}/{file}").as_str(),
            &config,
        )
        .unwrap();

        let file: String = fs::read_to_string(format!("./{folder}/{file}")).unwrap();
        assert_eq!(
            file,
            "config_file_path = '.'\ndebug_mode = 'error'\nsnapshots_path = '.'"
        );
    }

    #[test]
    #[serial]
    fn should_copy_folder_recurcivilly() {
        let Noisy { folder, cleanup: _ } = &Noisy::new();

        let config = Config {
            config_file_path: ".".to_string(),
            debug_mode: "error".to_string(),
            snapshots_path: ".".to_string(),
        };

        let file = "file.toml".to_string();

        search_file_create_config_folder_if_not_found(
            format!("./{folder}/{file}").as_str(),
            &config,
        )
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
    #[serial]
    fn should_copy_folder_recurcivilly_ignoring_files_of_setuprsignore() {
        let noisy = &mut Noisy::new()
            .add_file(NoisyFile {
                name: "normalfile".to_string(),
                content: "".to_string(),
            })
            .add_file(NoisyFile {
                name: "normalfile".to_string(),
                content: "".to_string(),
            })
            .add_file(NoisyFile {
                name: "ignored_file_0".to_string(),
                content: "".to_string(),
            })
            .add_folder("folder".to_string())
            .add_file(NoisyFile {
                name: "folder/ignored_file_1".to_string(),
                content: "".to_string(),
            });

        let folder = &noisy.folder.clone();
        Noisy::add_file_non_method(
            folder.to_owned(),
            NoisyFile {
                name: ".setuprsignore".to_string(),
                content: format!("{folder}/ignored_file_0\n{folder}/folder/ignored_file_1"),
            },
        );

        set_value(load_gitignore_patterns(Path::new(folder)));
        noisy.overwrite_cleanup(Box::new(move || {
            fs::remove_dir_all("test_folder_copy").unwrap();
        }));

        copy_dir_all(folder, "./test_folder_copy").unwrap();

        let on_folder = |file: &str| -> String { format!("./test_folder_copy/{file}") };

        assert!(Path::new(&on_folder("normalfile")).exists());
        assert!(!Path::new(&on_folder("ignored_file_0")).exists());
        assert!(!Path::new(&on_folder("folder/ignored_file_1")).exists());
    }

    #[test]
    #[serial]
    fn should_retrieve_id() {
        let Noisy { folder, cleanup: _ } = &Noisy::new();

        let config = Config {
            config_file_path: ".".to_string(),
            debug_mode: "error".to_string(),
            snapshots_path: ".".to_string(),
        };

        let file = "file.toml".to_string();

        search_file_create_config_folder_if_not_found(
            format!("./{folder}/{file}").as_str(),
            &config,
        )
        .unwrap();

        let result = get_all_snapshot_ids(folder).unwrap();
        let expected = vec![format!("{file}")];
        assert_eq!(result, expected);
    }
}
