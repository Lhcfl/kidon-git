use log::debug;

use crate::models::{
    object::{Object, Sha1Able},
    repo::WithRepo,
    stage::Stage,
    tree::{TreeLine, TreeLineKind},
};
use std::{collections::HashMap, fs, io, path::Path};

/// A wrapper for the stage, because you may add twice for the same file
pub struct FlattenTree {
    pub data: HashMap<String, TreeLine>,
    pub save_object: bool,
}

pub trait StageService<'a> {
    fn into_muter(self) -> WithRepo<'a, FlattenTree>;
}

impl<'a> StageService<'a> for WithRepo<'a, Stage> {
    fn into_muter(self) -> WithRepo<'a, FlattenTree> {
        WithRepo::new(
            self.repo,
            FlattenTree {
                data: self.unwrap().0.into_map(),
                save_object: true,
            },
        )
    }
}

impl<'a> WithRepo<'a, FlattenTree> {
    /// add file to the stage
    /// it WON'T save stage file (`.git/index`), until you save it.
    pub fn add_file(&mut self, path: &Path) -> io::Result<&mut Self> {
        let relative = path.strip_prefix(self.repo.working_dir()).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "{e}: root = {}, path = {}",
                    self.repo.working_dir().display(),
                    path.display()
                ),
            )
        })?;

        debug!("Adding file: {} ({})", relative.display(), path.display());
        let ctnt = fs::read(path)?;
        let blob = Object::Blob(
            String::from_utf8(ctnt)
                .map(|str| str.into())
                .unwrap_or_else(|e| e.into_bytes().into()),
        );

        let blob = self.wrap(blob);

        if self.save_object {
            blob.save()?;
        }

        let line = TreeLine {
            kind: TreeLineKind::File,
            name: relative
                .iter()
                .map(|part| part.to_string_lossy().into_owned())
                .collect::<Vec<String>>()
                .join("/"),
            sha1: blob.sha1().into(),
        };

        self.data.insert(line.name.clone(), line);

        Ok(self)
    }

    pub fn add_dir(&mut self, dir: &Path) -> io::Result<&mut Self> {
        if dir == self.repo.root {
            // skip the root directory
            return Ok(self);
        }
        if self.repo.ignores.contains(
            dir.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .as_ref(),
        ) {
            // skip the .gitignore s
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

    pub fn freeze(self) -> WithRepo<'a, Stage> {
        WithRepo::new(self.repo, Stage(self.unwrap().data.into()))
    }
}
