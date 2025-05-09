use log::debug;

use crate::models::{
    object::{Object, Sha1Able},
    repo::WithRepo,
    tree::{Tree, TreeLine, TreeLineKind},
};
use std::{collections::HashMap, fs, io, path::Path};

/// A wrapper for the tree, because you may add twice for the same file
pub struct MutableTree {
    pub data: HashMap<String, TreeLine>,
    pub save_object: bool,
}

impl<'a> WithRepo<'a, MutableTree> {
    fn debug_util(&self, path: &Path, debug_msg: &str) -> io::Result<()> {
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
        debug!("{} {} ({})", debug_msg, relative.display(), path.display());
        Ok(())
    }
    /// add file to the stage
    /// it WON'T save stage file (`.git/index`), until you save it.
    pub fn add_file(&mut self, path: &Path) -> io::Result<&mut Self> {
        let filename = path
            .file_name()
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("file name is invalid: {}", path.display()),
            ))?
            .to_string_lossy();

        self.debug_util(path, "Adding file")?;
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
            name: filename.to_string(),
            sha1: blob.sha1().into(),
        };

        self.data.insert(filename.to_string(), line);

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

        let mut tree = self.wrap(MutableTree {
            data: HashMap::new(),
            save_object: self.save_object,
        });

        for item in fs::read_dir(dir)? {
            tree.add_path(&item?.path())?;
        }

        let tree = tree.freeze().map(Object::Tree);
        let dirname = dir.file_name().unwrap_or_default().to_string_lossy();

        if self.save_object {
            tree.save()?;
        }

        self.data.insert(
            dirname.to_string(),
            TreeLine {
                kind: TreeLineKind::Tree,
                name: dirname.to_string(),
                sha1: tree.sha1().into(),
            },
        );

        Ok(self)
    }

    /// add a path to the stage
    pub fn add_path(&mut self, path: &Path) -> io::Result<&mut Self> {
        if path.is_file() {
            self.add_file(path)
        } else if path.is_dir() {
            if path == self.repo.working_dir() {
                for item in fs::read_dir(path)? {
                    self.add_path(&item?.path())?;
                }
                Ok(self)
            } else {
                self.add_dir(path)
            }
            // self.add_dir(path)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("path {} is not a file or directory", path.display()),
            ))
        }
    }

    pub fn freeze(self) -> WithRepo<'a, Tree> {
        WithRepo::new(self.repo, self.unwrap().data.into())
    }

    pub fn remove_dir(&mut self, path: &Path) -> io::Result<&mut Self> {
        let filename = path
            .file_name()
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("file name is invalid: {}", path.display()),
            ))?
            .to_string_lossy();

        if self.data.remove(&filename.to_string()).is_none() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("file {} not found", filename),
            ));
        }

        Ok(self)
    }
    pub fn remove_file(&mut self, path: &Path) -> io::Result<&mut Self> {
        let filename = path
            .file_name()
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("file name is invalid: {}", path.display()),
            ))?
            .to_string_lossy();

        self.debug_util(path, "Removing file")?;

        if self.data.remove(&filename.to_string()).is_none() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("file {} not found", filename),
            ));
        }

        Ok(self)
    }
    pub fn remove_path(&mut self, path: &Path) -> io::Result<&mut Self> {
        if path.is_file() {
            self.remove_file(path)
        } else if path.is_dir() {
            self.remove_dir(path)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("path {} is not a file or directory", path.display()),
            ))
        }
    }
}
