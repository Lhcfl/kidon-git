use std::{
    fs, io,
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
        fs::write(
            root.join(self.loaction()),
            serde_json::to_string(self)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        )
    }
    fn load(path: &Path) -> io::Result<Self> {
        let data = fs::read(path)?;
        serde_json::from_slice(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

pub trait Sha1Able {
    fn sha1(&self) -> String;
}
