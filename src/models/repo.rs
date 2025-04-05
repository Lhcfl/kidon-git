//! Repository Struct

use super::{head, object, refs};
use crate::traits::{Accessable, Accessor, DirContainer};
use crate::{models::head::Head, traits::Store};
use std::ops::Deref;
use std::{
    env, fs,
    io::{self, ErrorKind},
    path::PathBuf,
};

#[derive(Debug)]
pub struct Repository {
    pub root: PathBuf,
    head_: Head,
}

/// A wrapper for the storeable object with the repository path  
/// This is used to save and load the object from the repository  
/// Usage:
///   
/// ```rust
/// let head = repo.head(); // WithRepoPath<Head>
/// head.xxx = yyy; // Modify the head object
/// head.save(); // Save the object to the repository
/// ```
///
/// ```rust
/// let obj = repo.wrapped(Object::accessor("some sha1".into())); // WithRepoPath<Accessor<ObjectSha1, Object>>
/// obj.load(); // Load the object from the repository
/// ```
pub struct WithRepoPath<'r, T> {
    root: &'r PathBuf,
    inner: T,
}

impl<T> Deref for WithRepoPath<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> WithRepoPath<'_, T>
where
    T: Store,
{
    /// Save the storeable object to the repository
    pub fn save(&self) -> io::Result<()> {
        self.store(&self.root)
    }
}

impl<By, T> WithRepoPath<'_, Accessor<By, T>>
where
    T: Accessable<By>,
    T: Store,
{
    /// Load the storeable object from the repository
    pub fn load(&self) -> io::Result<WithRepoPath<T>> {
        let inner = T::load(&self.root.join(self.inner.path()))?;
        Ok(WithRepoPath {
            root: self.root,
            inner,
        })
    }
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
    pub fn wrapped<T>(&self, inner: T) -> WithRepoPath<T> {
        WithRepoPath {
            root: &self.root,
            inner,
        }
    }

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

        Ok(Repository {
            root: path,
            head_: head,
        })
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

    pub fn head(&self) -> WithRepoPath<&Head> {
        self.wrapped(&self.head_)
    }
}
