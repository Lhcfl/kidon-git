use std::io;

use crate::models::{branch::Branch, repo::Repository};

pub trait MergeSerivce {
  fn merge(&self, another_branch: &Branch) -> io::Result<()>;
}

impl MergeSerivce for Repository {
  /// Merge another branch into the current branch.
  ///
  /// This method will merge the specified branch into the current branch.
  /// It will handle conflicts and return an error if the merge fails.
  fn merge(&self, branch: &Branch) -> io::Result<()> {
    panic!("not implemented yet");
  }
}