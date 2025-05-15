use std::io;
use crate::models::{repo::Repository, tree::Tree};

trait DumpTreeService {
    /// Dump the tree to the working directory
    fn dump_tree(&self, tree: &Tree) -> io::Result<()>;
}

impl DumpTreeService for Repository {
    fn dump_tree(&self, tree: &Tree) -> io::Result<()> {
      panic!("todo");
    }
}