use std::fmt::Debug;
use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error)]
pub enum Error {
    #[error("Unknown Error")]
    Unknown,

    #[error("{0}")]
    IOError(#[from] io::Error),
}

impl Error {
    pub fn into_message(&self) -> String {
        match self {
            Error::Unknown => "Unknown Error".to_string(),
            Error::IOError(_) => "IO error".to_string(),
            // Error::IOError(e) => match e.kind() {
            //     io::ErrorKind::ExecutableFileBusy => "Nicolas".to_string(),
            //     _ => "REST".to_string(),
            // },
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // error!("{self}");
        writeln!(f, "{}", self)?;
        Ok(())
    }
}
