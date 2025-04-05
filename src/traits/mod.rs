use std::{
    fs, io,
    marker::PhantomData,
    ops::Deref,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

pub trait DirContainer {
    const DIRECTORY: &'static str;

    fn make_dir(root: &Path) -> io::Result<()> {
        let path = root.join(Self::DIRECTORY);
        std::fs::create_dir_all(path)
    }

    fn check_dir_exists(root: &Path) -> bool {
        let path = root.join(Self::DIRECTORY);
        path.exists()
    }
}

pub trait Store
where
    Self: Serialize,
    Self: for<'de> Deserialize<'de>,
{
    fn loaction(&self) -> PathBuf;
    fn store(&self, root: &Path) -> io::Result<()> {
        let path = root.join(self.loaction());
        if let Some(parent) = path.parent() {
            // Safely ignores the error if the directory already exists
            let _ = fs::create_dir_all(parent);
        }
        fs::write(
            path,
            serde_json::to_string(self)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        )
    }
    fn load(path: &Path) -> io::Result<Self> {
        let data = fs::read(path)?;
        let inner = serde_json::from_slice(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(inner)
    }
}

/// Wraped the accessor to the storeable object
pub struct Accessor<By, T>
where
    T: Store,
    T: Accessable<By>,
{
    by: By,
    _will_into: PhantomData<T>,
}

/// The trait for the object that can be accessed by the accessor
pub trait Accessable<By>
where
    Self: Store,
{
    /// Get an accessor of the object
    fn accessor(by: impl Into<By>) -> Accessor<By, Self> {
        Accessor {
            by: by.into(),
            _will_into: PhantomData,
        }
    }

    fn path_of(by: &By) -> PathBuf;
}

impl<By, T> Accessor<By, T>
where
    T: Store,
    T: Accessable<By>,
{
    pub fn path(&self) -> PathBuf {
        T::path_of(&self.by)
    }
}
