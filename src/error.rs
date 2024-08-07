use std::fmt::Debug;
use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error)]
pub enum Error {
    #[error("{0}")]
    TomlError(#[from] toml::de::Error),

    #[error("Missing setuprs init files, please run setuprs init")]
    MissingBasicInitialization,

    #[error("{0}")]
    IOError(#[from] io::Error),

    #[error("Provided snapshot don't exist")]
    SnapshotDontExist,
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // error!("{self}");
        writeln!(f, "{}", self)?;
        Ok(())
    }
}
