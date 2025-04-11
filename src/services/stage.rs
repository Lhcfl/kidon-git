use crate::models::{
    object::{Object, Sha1Able},
    repo::WithRepoPath,
    stage::Stage,
    tree::{TreeLine, TreeLineKind},
};
use std::{collections::HashMap, fs, io, path::Path};

/// A wrapper for the stage, because you may add twice for the same file
pub struct StageMuter {
    pub data: HashMap<String, TreeLine>,
}

pub trait StageService<'a> {
    fn into_muter(self) -> WithRepoPath<'a, StageMuter>;
}

impl<'a> StageService<'a> for WithRepoPath<'a, Stage> {
    fn into_muter(self) -> WithRepoPath<'a, StageMuter> {
        WithRepoPath::new(
            self.root,
            StageMuter {
                data: self.unwrap().0.into_map(),
            },
        )
    }
}

impl<'a> WithRepoPath<'a, StageMuter> {
    /// add file to the stage
    /// it WON'T save stage file (`.git/index`), until you save it.
    pub fn add_file(&mut self, path: &Path) -> io::Result<&mut Self> {
        let relative = path.strip_prefix(self.working_dir()).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "{e}: root = {}, path = {}",
                    self.working_dir().display(),
                    path.display()
                ),
            )
        })?;
        println!("Adding file: {} ({})", relative.display(), path.display());
        let ctnt = fs::read(path)?;
        let blob = Object::Blob(
            String::from_utf8(ctnt)
                .map(|str| str.into())
                .unwrap_or_else(|e| e.into_bytes().into()),
        );

        let blob = self.wrap(blob);
        blob.save()?;

        self.data.insert(
            relative.to_string_lossy().into(),
            TreeLine {
                kind: TreeLineKind::File,
                name: relative.to_string_lossy().into(),
                sha1: blob.sha1().into(),
            },
        );

        Ok(self)
    }

    pub fn add_dir(&mut self, dir: &Path) -> io::Result<&mut Self> {
        if dir == self.root {
            // skip the root directory
            return Ok(self);
        }
        for item in fs::read_dir(dir)? {
            self.add_path(&item?.path())?;
        }
        Ok(self)
    }

    /// add a path to the stage
    pub fn add_path(&mut self, path: &Path) -> io::Result<&mut Self> {
        if path.is_file() {
            self.add_file(path)
        } else if path.is_dir() {
            self.add_dir(path)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("path {} is not a file or directory", path.display()),
            ))
        }
    }

    pub fn freeze(self) -> WithRepoPath<'a, Stage> {
        WithRepoPath::new(self.root, Stage(self.unwrap().data.into()))
    }
}
