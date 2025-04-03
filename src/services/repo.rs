//! Repository services

use std::{
    fs,
    io::{self, ErrorKind},
    path::Path,
};

pub struct Repository {
    pub path: &'static Path,
}

impl Repository {}

pub enum RepositoryInitError {
    NotInitialized,
    NotADirectory,
    UnknownError(ErrorKind),
}

impl From<io::Error> for RepositoryInitError {
    fn from(value: io::Error) -> Self {
        Self::UnknownError(value.kind())
    }
}

/// Load the repository form .git folder
/// TODO
pub fn load() -> Result<Repository, RepositoryInitError> {
    let dir = Path::new(".git");
    let repo = match fs::read_dir(dir).map_err(|e| e.kind()) {
        Ok(x) => x,
        Err(ErrorKind::NotADirectory) => return Err(RepositoryInitError::NotADirectory),
        Err(ErrorKind::NotFound) => return Err(RepositoryInitError::NotInitialized),
        Err(e) => return Err(RepositoryInitError::UnknownError(e)),
    };

    for item in repo {
        println!("find: {:?}", item?.file_name());
    }

    Ok(Repository { path: dir })
}
