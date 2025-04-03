use std::{
    fs, io,
    path::{Path, PathBuf},
};

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

pub trait SerDe
where
    Self: Sized,
{
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: impl Into<Vec<u8>>) -> Result<Self, String>;
}

pub trait Store
where
    Self: SerDe,
{
    fn loaction(&self) -> PathBuf;
    fn store(&self, root: &Path) -> io::Result<()> {
        fs::write(root.join(self.loaction()), self.serialize())
    }
    fn load(path: &Path) -> io::Result<Self> {
        let data = fs::read(path)?;
        Self::deserialize(data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}
