//! Repository services

use super::{head, object, refs};
use crate::traits::DirContainer;
use crate::{models::head::Head, traits::Store};
use std::{
    env, fs,
    io::{self, ErrorKind},
    path::PathBuf,
};

#[derive(Debug)]
pub struct Repository {
    pub root: PathBuf,
    pub head: Head,
}

impl DirContainer for Repository {
    const DIRECTORY: &str = ".kidon-git";
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum RepositoryInitError {
    /// The repository is not initialized
    NotInitialized,
    /// The dir is not a git repository, or is broken
    BadGitRepositoryDir,
    /// Some of subdirectories are not initialized
    NotADirectory(io::Error),
    /// Some of directory already exists
    AlreadyExists(io::Error),
    /// Unknown error
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

impl Repository {
    fn find_root() -> PathBuf {
        env::current_dir().expect("The currecnt directory is not valid")
    }

    /// Load the repository form .git folder
    /// TODO
    pub fn load() -> Result<Self, RepositoryInitError> {
        let path = Self::find_root().join(Self::DIRECTORY);
        let dir = fs::read_dir(&path)?;

        for item in dir {
            println!("find: {:?}", item?.file_name());
        }

        refs::Refs::check_dir_exists(&path);
        object::Object::check_dir_exists(&path);

        let head = head::Head::load(&path.join("HEAD"))?;

        Ok(Repository { root: path, head })
    }

    /// Initialize the repository
    /// TODO
    pub fn init() -> Result<Self, RepositoryInitError> {
        let path = Self::find_root().join(Self::DIRECTORY);

        Self::make_dir(&Self::find_root())?;
        refs::Refs::make_dir(&path)?;
        object::Object::make_dir(&path)?;

        let head = head::Head {
            kind: head::HeadKind::Local,
            branch: "main".to_string(),
        };
        head.store(&path)?;

        Ok(Self::load()?)
    }
}
