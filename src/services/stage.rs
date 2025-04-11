use crate::models::{
    object::{Object, Sha1Able},
    repo::WithRepoPath,
    stage::Stage,
    tree::{TreeLine, TreeLineKind},
};
use std::{fs, io, path::Path};

pub trait StageService {
    fn add_file(&mut self, path: &Path) -> io::Result<&mut Self>;
    fn add_dir(&mut self, dir: &Path) -> io::Result<&mut Self>;

    /// add a path to the stage
    fn add_path(&mut self, path: &Path) -> io::Result<&mut Self> {
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
}

impl StageService for WithRepoPath<'_, Stage> {
    /// add file to the stage
    /// it WON'T save stage file (`.git/index`), until you save it.
    fn add_file(&mut self, path: &Path) -> io::Result<&mut Self> {
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

        self.objects.push(TreeLine {
            kind: TreeLineKind::File,
            name: relative.to_string_lossy().into(),
            sha1: blob.sha1().into(),
        });

        Ok(self)
    }

    fn add_dir(&mut self, dir: &Path) -> io::Result<&mut Self> {
        for item in fs::read_dir(dir)? {
            self.add_path(&item?.path())?;
        }
        Ok(self)
    }
}
