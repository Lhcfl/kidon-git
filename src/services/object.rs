//! git objects

use std::{fs::File, io, path::Path};

use super::repo::Repository;

const OBJECTS_DIR: &str = "objects";

pub struct Object {
    pub sha1: String,
    pub file: File,
}

impl Object {
    /// load the object from filesystem
    pub fn load_from_sha1(repo: &Repository, sha1: impl Into<String>) -> io::Result<Self> {
        let sha1 = sha1.into();
        let path = repo.path.join(OBJECTS_DIR).join(&sha1);
        let file = std::fs::File::open(path)?;
        let object = Self { sha1, file };
        Ok(object)
    }
}

pub fn init_dir(path: &Path) -> io::Result<()> {
    let objects_path = path.join(OBJECTS_DIR);
    std::fs::create_dir_all(objects_path)?;
    Ok(())
}
