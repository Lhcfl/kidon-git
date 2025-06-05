use crate::{
    models::{Accessible, object::Object, repo::Repository, tree::Tree},
    services::tree::{ComparedKind, compare_trees},
};
use std::io;

pub trait DumpTreeService {
    /// Dump the tree to the working directory
    fn dump_tree(&self, tree: Tree) -> io::Result<()>;
}

impl DumpTreeService for Repository {
    fn dump_tree(&self, target_tree: Tree) -> io::Result<()> {
        let current_branch = self.head().load_branch()?;

        let current_commit = current_branch.get_current_commit()?;
        let current_tree = current_commit.get_tree()?;

        let changes = compare_trees(&current_tree, &self.wrap(target_tree))?;

        for change in changes {
            match change.kind {
                ComparedKind::Added | ComparedKind::Modified => {
                    // Write new or modified files
                    let blob = self
                        .wrap(Object::accessor(&change.line.sha1))
                        .load()?
                        .clone()
                        .cast_blob();
                    // Emmm.. assuming workign dir is .git's parent @lhcfl maybe add pwd root in repo?
                    let path = self.working_dir().join(&change.line.name);

                    // Ensure parent directories exist
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }

                    // Write the file
                    std::fs::write(&path, blob.as_bytes())?;
                }
                ComparedKind::Deleted => {
                    // Remove deleted files
                    let path = self.working_dir().join(&change.line.name);

                    if path.is_file() || path.is_symlink() {
                        std::fs::remove_file(&path)?;
                    } else if path.is_dir() {
                        std::fs::remove_dir_all(&path)?;
                    }
                }
            }
        }

        Ok(())
    }
}
