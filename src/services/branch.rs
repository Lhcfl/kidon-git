use crate::{
    models::{
        branch::Branch, object::Object, repo::{Repository, WithRepo}, stage::Stage
    }, services::{stage::StageService, tree::{compare_trees, ComparedKind}}, models::{Accessable, DirContainer}
};
use std::io;

pub enum BranchCreationError {
    AlreadyExists,
    InvalidName,
    IoError(io::Error),
}
impl From<io::Error> for BranchCreationError {
    fn from(err: io::Error) -> Self {
        BranchCreationError::IoError(err)
    }
}
impl From<BranchCreationError> for anyhow::Error {
    fn from(err: BranchCreationError) -> Self {
        match err {
            BranchCreationError::AlreadyExists => anyhow::anyhow!("branch already exists"),
            BranchCreationError::InvalidName => anyhow::anyhow!("invalid branch name"),
            BranchCreationError::IoError(err) => err.into(),
        }
    }
}
pub trait BranchService {
    fn list_branch(&self) -> io::Result<Vec<String>>;
    fn create_branch(&self, branch_name: &str)
    -> Result<WithRepo<'_, Branch>, BranchCreationError>;
    fn delete_branch(&self, branch_name: &str) -> io::Result<()>;
    fn branch_exists(&self, branch_name: &str) -> io::Result<bool>;
    fn checkout_branch(&self, branch_name: &str) -> io::Result<()>;
}

impl BranchService for Repository {
    /// list branch names, including remote branches, by a vector of strings
    fn list_branch(&self) -> io::Result<Vec<String>> {
        let mut branches = Vec::new();
        for entry in std::fs::read_dir(self.root.join(Branch::DIRECTORY).join("heads"))? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let branch_name = entry.file_name().into_string().map_err(|s| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("branch name {s:?} is not valid"),
                    )
                })?;
                branches.push(branch_name);
            }
        }

        for entry in std::fs::read_dir(self.root.join(Branch::DIRECTORY).join("remotes"))? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let remote_name = entry.file_name().into_string().map_err(|s| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("remote name {s:?} is not valid"),
                    )
                })?;
                for branch in std::fs::read_dir(entry.path())? {
                    let branch = branch?;
                    if branch.file_type()?.is_file() {
                        let branch_name = branch.file_name().into_string().map_err(|s| {
                            io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!("branch name {s:?} is not valid"),
                            )
                        })?;
                        branches.push(format!("{remote_name}/{branch_name}"));
                    }
                }
            }
        }

        Ok(branches)
    }

    /// Create a new branch with the given name based on the current branch
    fn create_branch(&self, name: &str) -> Result<WithRepo<'_, Branch>, BranchCreationError> {
        Branch::validate_name(name)
            .then_some(())
            .ok_or(BranchCreationError::InvalidName)?;
        let Err(_) = self.wrap(Branch::accessor(&name)).load() else {
            return Err(BranchCreationError::AlreadyExists);
        };
        let wrap = self.wrap(Branch {
            remote: None,
            name: name.to_string(),
            head: self.head().branch().load()?.unwrap().head,
        });
        wrap.save()?;
        Ok(wrap)
    }
    fn delete_branch(&self, name: &str) -> io::Result<()> {
        let branch = self.wrap(Branch::accessor(&name));
        let branch = branch.load()?;
        if branch.full_name() == self.head().branch_name {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot delete current branch",
            ));
        }
        branch.remove()?;
        Ok(())
    }

    fn branch_exists(&self, name: &str) -> io::Result<bool> {
        Ok(self.list_branch()?.iter().any(|b| b == name))
    }

    fn checkout_branch(&self, name: &str) -> io::Result<()> {
        // Step 1: Check if the branch exists
        if !self.branch_exists(name)? {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("branch '{name}' does not exist"),
            ));
        }

        // Step 2: Load the branch
        let branch = self.wrap(Branch::accessor(&name)).load()?;

        // Step 3: Check if the branch has any commits
        let target_commit_sha1 = branch.head.as_ref().unwrap();

        // Step 4: Load the target branch's tree
        let target_commit = self.wrap(Object::accessor(target_commit_sha1)).load()?.map(|t| t.cast_commit());
        let target_tree = self.wrap(Object::accessor(&target_commit.tree)).load()?.map(|t| t.cast_tree());
        let target_tree=self.wrap(target_tree);

        // Step 5: Load the current branch's tree
        let binding = self.head().branch().load()?;
        let current_commit_sha1 = binding.head.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "current branch has no commits",
            )
        })?;
        let current_commit = self.wrap(Object::accessor(current_commit_sha1)).load()?.map(|t| t.cast_commit());
        let current_tree = self.wrap(Object::accessor(&current_commit.tree)).load()?.map(|t| t.cast_tree());
        let current_tree=self.wrap(current_tree);

        // Step 6: Compare the trees
        let changes = compare_trees(&current_tree, &target_tree)?;

        // Step 7: Apply changes to the working directory
        let mut stage = self.stage()?.into_muter();
        for change in changes {
            match change.kind {
                ComparedKind::Added | ComparedKind::Modified => {
                    // Write new or modified files
                    let blob = self
                        .wrap(Object::accessor(&change.line.sha1))
                        .load()?.clone()
                        .cast_blob();
                    let path = self.root.join(&change.line.name);

                    // Ensure parent directories exist
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }

                    // Write the file
                    std::fs::write(&path, blob.as_bytes())?;
                    stage.add_path(&path)?;
                }
                ComparedKind::Deleted => {
                    // Remove deleted files
                    let path = self.root.join(&change.line.name);
                    if path.is_file() || path.is_symlink() {
                        std::fs::remove_file(&path)?;
                    } else if path.is_dir() {
                        std::fs::remove_dir_all(&path)?;
                    }

                    stage.remove_path(&path)?;
                }
            }
        }
        stage.freeze().map(Stage).save()?;

        // Step 8: Update HEAD to point to the new branch
        let mut head = self.head().clone();
        head.branch_name = branch.full_name();
        let head = self.wrap(head);
        head.save()?;

        println!("Switched to branch '{name}'");
        Ok(())
    }
}

