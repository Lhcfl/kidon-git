use std::{
    fs, io,
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
    fn load(path: &Path) -> io::Result<WithLocation<Self>> {
        let data = fs::read(path)?;
        let inner = serde_json::from_slice(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(WithLocation {
            location: path.to_path_buf(),
            inner,
        })
    }
}

pub struct WithLocation<T: Store> {
    location: PathBuf,
    pub inner: T,
}

impl<T: Store> WithLocation<T> {
    fn save(&self) -> io::Result<()> {
        fs::write(
            &self.location,
            serde_json::to_string(&self.inner)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        )
    }
}

impl<T: Store> Deref for WithLocation<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
