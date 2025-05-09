use std::ops::Deref;

use crate::{
    models::{
        commit,
        object::{Object, Sha1Able},
        repo::Repository,
    },
    services::tree::compare_trees,
    traits::Accessable,
};
use chrono::Utc;
use clap::Args;

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
        let mut branch = repo.head().branch().load()?;

        // Step 1: Ensure commit message is provided
        let message = self
            .message
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("commit message is required"))?;

        // Step 2: Generate tree object from the stage
        let stage = repo.stage()?;
        let tree = stage.map(|s| s.0);

        // Step 3: Compare the tree with the current HEAD, to check if working tree clean
        let compared = if let Some(current_sha1) = branch.head.as_ref() {
            let current_commit = repo.wrap(Object::accessor(current_sha1)).load()?;
            let current_tree_sha1 = current_commit.unwrap().cast_commit().tree;
            let current_tree = repo
                .wrap(Object::accessor(&current_tree_sha1))
                .load()?
                .map(|t| t.cast_tree());

            let compared = compare_trees(&current_tree, &tree)?;
            if compared.is_empty() {
                println!("nothing to commit, working tree clean");
                return Ok(());
            }
            Some(compared)
        } else {
            None
        };

        // Step 4: Create commit object
        let tree = tree.map(Object::Tree);
        tree.save()?;
        let commit = commit::Commit::new(commit::CommitBuilder {
            tree: tree.sha1().into(),
            parent: branch.head.clone(),
            message: message.to_string(),
        });

        let commit_sha1 = commit.sha1();
        repo.wrap(Object::Commit(commit)).save()?;

        // Step 5: Update branch HEAD
        branch.head = Some(commit_sha1.clone().into());
        branch.save()?;

        let tip = if branch.head.is_some() {
            " (root-commit) "
        } else {
            " "
        };

        println!("[{}{tip}{}] {message}", branch.name, &commit_sha1[0..6]);

        // Log commit status
        if let Some(compared) = compared {
            println!("{} file changed", compared.len());
            for line in compared {
                println!("{line}");
            }
        }

        Ok(())
    }
}
