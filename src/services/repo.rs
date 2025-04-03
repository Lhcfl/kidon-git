//! Repository services

const GIT_DIR: &str = ".kidon-git";

use std::{
    fs,
    io::{self, ErrorKind},
    path::Path,
};

use super::{object, refs};

#[derive(Debug)]
pub struct Repository {
    pub path: &'static Path,
}

impl Repository {}

#[derive(Debug)]
#[allow(dead_code)]
pub enum RepositoryInitError {
    NotInitialized,
    NotADirectory(io::Error),
    AlreadyExists(io::Error),
    UnknownError(io::Error),
}

impl From<io::Error> for RepositoryInitError {
    fn from(value: io::Error) -> Self {
        match value.kind() {
            ErrorKind::AlreadyExists => RepositoryInitError::AlreadyExists(value),
            ErrorKind::NotADirectory => RepositoryInitError::NotADirectory(value),
            ErrorKind::NotFound => RepositoryInitError::NotInitialized,
            _ => RepositoryInitError::UnknownError(value),
        }
    }
}

/// Load the repository form .git folder
/// TODO
pub fn load() -> Result<Repository, RepositoryInitError> {
    let dir = Path::new(GIT_DIR);
    let repo = fs::read_dir(dir)?;

    for item in repo {
        println!("find: {:?}", item?.file_name());
    }

    Ok(Repository { path: dir })
}

/// Initialize the repository
/// TODO
pub fn init() -> Result<Repository, RepositoryInitError> {
    let dir = Path::new(GIT_DIR);

    fs::create_dir(dir)?;
    refs::init_dir(dir)?;
    object::init_dir(dir)?;

    Ok(load()?)
}
