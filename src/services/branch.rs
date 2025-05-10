use crate::{
    models::{
        branch::Branch,
        repo::{Repository, WithRepo}, tree::Tree,
        object::{Object},
        tree::TreeLineKind
    },
    traits::{Accessable, DirContainer},
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
    fn clear_working_directory(&self) -> io::Result<()>;
    fn apply_tree_to_working_directory(&self, tree: &Tree) -> io::Result<()>;
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
                format!("branch '{}' does not exist", name),
            ));
        }

        // Step 2: Load the branch
        let branch = self.wrap(Branch::accessor(&name)).load()?;

        // Step 3: Load the tree object of the branch's HEAD
        let head_commit_sha1 = branch.head.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "branch has no commits")
        })?;
        let head_commit = self.wrap(Object::accessor(head_commit_sha1)).load()?.clone();
        let tree_sha1 = head_commit.cast_commit().tree;
        let tree = self.wrap(Object::accessor(&tree_sha1)).load()?.clone().cast_tree();

        // Step 4: Clear the working directory
        self.clear_working_directory()?;

        // Step 5: Apply the tree to the working directory
        self.apply_tree_to_working_directory(&tree)?;

        // Step 6: Update HEAD to point to the new branch
        let mut head = self.head().clone();
        head.branch_name = branch.full_name();
        let head = self.wrap(head); // Save the modified Head object
        head.save()?;

        println!("Switched to branch '{}'", name);
        Ok(())
    }
    fn clear_working_directory(&self) -> io::Result<()> {
        for entry in std::fs::read_dir(&self.root)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() || path.is_symlink() {
                std::fs::remove_file(path)?;
            } else if path.is_dir() {
                std::fs::remove_dir_all(path)?;
            }
        }
        Ok(())
    }
    fn apply_tree_to_working_directory(&self, tree: &Tree) -> io::Result<()> {
        for line in &tree.objects {
            let path = self.root.join(&line.name);

            match line.kind {
                TreeLineKind::File | TreeLineKind::Executable => {
                    // Step 1: Ensure parent directories exist
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }

                    // Step 2: Check if the file already exists
                    if path.exists() {
                        println!("File '{}' already exists, overwriting.", path.display());
                    }

                    // Step 3: Write the file content to the working directory
                    let blob = self
                        .wrap(Object::accessor(&line.sha1))
                        .load()?.clone()
                        .cast_blob();
                    std::fs::write(&path, blob.as_bytes()).map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            format!("Failed to write file '{}': {}", path.display(), e),
                        )
                    })?;
                }
                TreeLineKind::Symlink => {
                    // Step 4: Handle symbolic links
                    let blob = self
                        .wrap(Object::accessor(&line.sha1))
                        .load()?.clone()
                        .cast_blob();
                    let target = std::str::from_utf8(blob.as_bytes()).map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Invalid symlink target for '{}': {}", path.display(), e),
                        )
                    })?;
                    if path.exists() {
                        std::fs::remove_file(&path)?; // Remove existing file or symlink
                    }
                    std::os::unix::fs::symlink(target, &path).map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            format!("Failed to create symlink '{}': {}", path.display(), e),
                        )
                    })?;
                }
                TreeLineKind::Tree => {
                    // Step 5: Recursively apply subtrees
                    let subtree = self
                        .wrap(Object::accessor(&line.sha1))
                        .load()?.clone()
                        .cast_tree();
                    self.apply_tree_to_working_directory(&subtree)?;
                }
            }
        }
        Ok(())
    }
}

