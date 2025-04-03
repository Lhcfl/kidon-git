//! git objects

use std::{fs::File, io};

use crate::traits::DirContainer;

use super::repo::Repository;

pub struct Object {
    pub sha1: String,
    pub file: File,
}

impl DirContainer for Object {
    const DIRECTORY: &'static str = "objects";
}

impl Object {
    /// load the object from filesystem
    pub fn load_from_sha1(repo: &Repository, sha1: impl Into<String>) -> io::Result<Self> {
        let sha1 = sha1.into();
        let path = repo.path.join(Self::DIRECTORY).join(&sha1);
        let file = std::fs::File::open(path)?;
        let object = Self { sha1, file };
        Ok(object)
    }
}
