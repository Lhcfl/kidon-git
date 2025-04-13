use std::{
    collections::HashSet,
    fs, io,
    ops::Deref,
    path::{Path, PathBuf},
};

use crate::traits::Store;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ignores(HashSet<String>);

impl Deref for Ignores {
    type Target = HashSet<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Store for Ignores {
    fn location(&self) -> PathBuf {
        PathBuf::from(".gitignore")
    }
    fn store(&self, _file: &Path) -> io::Result<()> {
        panic!("cannot save ignores");
    }
    fn load(root: &Path) -> io::Result<Self> {
        let file = fs::read(
            root.parent()
                .expect("git repo should have parent")
                .join(".gitignore"),
        );

        let mut ret = HashSet::new();
        ret.insert(".git".to_string());

        let Ok(ctnt) = file else {
            return Ok(Ignores(ret));
        };

        String::from_utf8_lossy(&ctnt)
            .lines()
            .filter(|line| !line.is_empty())
            .for_each(|line| {
                ret.insert(line.to_string());
            });

        Ok(Ignores(ret))
    }
}
