//! Repository, the entry of everything

use super::branch::Branch;
use super::ignores::Ignores;
use super::stage::Stage;
use super::{branch, head, object};
use crate::traits::{Accessable, Accessor, DirContainer};
use crate::{models::head::Head, traits::Store};
use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::{
    env, fs,
    io::{self, ErrorKind},
    path::PathBuf,
};

/// The repository is the entry point to the internal structure of git. Almost
/// all operations, whether loading objects from disk or saving them, need to
/// first obtain the repository structure and find out the location of the .git
/// folder from the root of the repository structure.
#[derive(Debug)]
pub struct Repository {
    /// .git dir for the repository
    pub root: PathBuf,
    pub ignores: Ignores,
    head_: Head,
}

/// A wrapper, like a [Box] but not dynamic, for any object to store the
/// referance with the repository  
/// This is designed to use in saving and loading the object from the repository  
///
/// # Usage:
///   
/// ```rust
/// let head = repo.head(); // WithRepoPath<Head>
/// head.xxx = yyy; // Modify the head object
/// head.save(); // Save the object to the repository
/// ```
///
/// ```rust
/// let obj = repo.wrap(Object::accessor("some sha1".into())); // WithRepoPath<Accessor<ObjectSha1, Object>>
/// obj.load(); // Load the object from the repository
/// ```
///
/// ```rust
/// let blob = repo.wrap(Object::Blob("some blob".into())); // WithRepoPath<Blob>
/// blob.save(); // Save the object to the repository
/// ```
pub struct WithRepo<'r, T> {
    pub repo: &'r Repository,
    inner: T,
}

impl<'r, T> WithRepo<'r, T> {
    pub fn new(repo: &'r Repository, inner: T) -> Self {
        WithRepo { repo, inner }
    }

    /// Wrap the storeable object with the repository path
    pub fn wrap<To>(&self, inner: To) -> WithRepo<'r, To> {
        WithRepo {
            repo: self.repo,
            inner,
        }
    }

    /// unpack self, and get the inner object
    pub fn unwrap(self) -> T {
        self.inner
    }

    /// A `WithRepo<T>` is actually a
    /// [Functor](https://www.adit.io/posts/2013-04-17-functors,_applicatives,_and_monads_in_pictures.html), you can apply a T -> U function to `WithRepo<T>` to get` WithRepo<U>` by mapping it.
    ///
    /// # Example
    ///
    /// ```rust
    /// let a = repo.wrap(1);
    /// let b = a.map(|x| x + 1);
    /// /// then b.unwrap() == 2
    /// ```
    ///
    /// referance: [std::option::Option::map]
    pub fn map<F, U>(self, f: F) -> WithRepo<'r, U>
    where
        F: FnOnce(T) -> U,
    {
        WithRepo {
            repo: self.repo,
            inner: f(self.inner),
        }
    }
}

impl<T> Deref for WithRepo<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for WithRepo<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'r, T: Clone> WithRepo<'r, T> {
    /// Clone the inner object, and returns the wrapped object
    pub fn cloned(&self) -> WithRepo<'r, T> {
        WithRepo {
            repo: self.repo,
            inner: self.inner.clone(),
        }
    }
}

impl<T> Display for WithRepo<'_, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<T> WithRepo<'_, T>
where
    T: Store,
{
    /// Save the storeable object to disk
    pub fn save(&self) -> io::Result<()> {
        self.store(&self.repo.root)
    }
    /// Delete the storeable object from disk
    pub fn remove(&self) -> io::Result<()> {
        self.delete(&self.repo.root)
    }
}

impl<'r, By, T> WithRepo<'r, Accessor<'_, By, T>>
where
    T: Accessable<By>,
    T: Store,
{
    /// Load the storeable object from disk
    pub fn load(&self) -> io::Result<WithRepo<'r, T>> {
        let inner = T::load(&self.repo.root.join(self.inner.path()))?;
        Ok(WithRepo {
            repo: self.repo,
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

impl Display for RepositoryInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryInitError::NotInitialized => write!(
                f,
                "fatal: not a git repository (or any of the parent directories)"
            ),
            RepositoryInitError::BadGitRepositoryDir => {
                write!(f, "fatal: The dir is not a git repository, or is broken")
            }
            RepositoryInitError::NotADirectory(e) => {
                write!(
                    f,
                    "fatal: The dir cannot be a valid git repository since some file already exists: {e}"
                )
            }
            RepositoryInitError::AlreadyExists(e) => {
                write!(
                    f,
                    "fatal: The dir cannot be a valid git repository since some directory already exists: {e}"
                )
            }
            RepositoryInitError::UnknownError(e) => {
                write!(f, "fatal: unknown error: {e}")
            }
        }
    }
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

impl From<RepositoryInitError> for anyhow::Error {
    fn from(value: RepositoryInitError) -> Self {
        anyhow::anyhow!(value.to_string())
    }
}

impl Repository {
    pub fn wrap<T>(&self, inner: T) -> WithRepo<T> {
        WithRepo { repo: self, inner }
    }

    // TODO find the root of the git repository recursively, because you may
    // execute the command inside a subdirectory of the repository
    fn find_root() -> PathBuf {
        env::current_dir().expect("The currecnt directory is not valid")
    }

    /// working dir of the git repo
    pub fn working_dir(&self) -> &Path {
        self.root
            .parent()
            .expect(".git directory should never be the root")
    }

    /// Load the repository form .git folder
    pub fn load() -> Result<Self, RepositoryInitError> {
        let path = Self::find_root().join(Self::DIRECTORY);
        let _ = fs::read_dir(&path)?;

        branch::Branch::check_dir_exists(&path);
        object::Object::check_dir_exists(&path);

        let head = head::Head::load(&path.join("HEAD"))?;

        Ok(Repository {
            ignores: Ignores::load(&path)?,
            root: path,
            head_: head,
        })
    }

    /// Initialize the repository
    pub fn init() -> Result<Self, RepositoryInitError> {
        let path = Self::find_root().join(Self::DIRECTORY);

        Self::make_dir(&Self::find_root())?;
        branch::Branch::make_dir(&path)?;
        object::Object::make_dir(&path)?;

        let head = head::Head {
            kind: head::HeadKind::Local,
            branch_name: "main".to_string(),
        };
        head.store(&path)?;

        let main_branch = Branch {
            remote: None,
            name: "main".to_string(),
            head: None,
        };
        main_branch.store(&path)?;

        Self::load()
    }

    /// get the head of the repository
    pub fn head(&self) -> WithRepo<&Head> {
        self.wrap(&self.head_)
    }

    /// get the staging index of the repository
    pub fn stage(&self) -> io::Result<WithRepo<Stage>> {
        let stage_file = self.root.join(Stage::LOCATION);
        Ok(if stage_file.is_file() {
            self.wrap(Stage::load(&stage_file)?)
        } else {
            self.wrap(Stage::empty())
        })
    }
}
