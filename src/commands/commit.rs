use chrono::Utc;
use clap::Args;
use crate::models::{
    object::Object, 
    repo::Repository,
    object::Sha1Able,
    commit,
};

use super::Exec;

#[derive(Debug, Args)]
pub struct Commit {
    /// commit message
    #[arg(short, long)]
    pub message: Option<String>,
}

impl Exec for Commit {
    fn exec(&self) -> anyhow::Result<()> {
        let repo = Repository::load()?;

        // Step 2: Ensure commit message is provided
        let message = self
            .message
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("commit message is required"))?;

        // Step 3: Generate tree object from the stage
        let stage = repo.stage()?;
        let tree = stage.map(|s| s.0);
        let tree_sha1 = tree.sha1();
        tree.map(|s| Object::Tree(s)).save()?;

        let mut branch = repo.head().branch().load()?;
        // TODO: check this
        // Step 4: Create commit object
        let parent = branch.head.clone();
        let commit = commit::Commit {
            tree: tree_sha1.into(),
            parent,
            timestamp: (Utc::now().timestamp(), Utc::now().timestamp_subsec_nanos()),
            message: message.to_string(),
        };
        let commit_sha1 = commit.sha1();
        repo.wrap(Object::Commit(commit)).save()?;

        // Step 5: Update branch HEAD
        branch.head = Some(commit_sha1.into());
        branch.save()?;

        println!("[{}] {}", branch.name, message);
        Ok(())
    }
}
