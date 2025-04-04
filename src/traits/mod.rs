use std::{
    fs, io,
    path::{Path, PathBuf},
};

use enum_dispatch::enum_dispatch;
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
        serde_json::from_slice(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}
