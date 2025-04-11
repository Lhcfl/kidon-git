use std::ops::Not;

use clap::Args;
use colored::Colorize;

use crate::{
    models::{object::Object, repo::Repository},
    services::tree::{ComparedKind, compare_trees},
    traits::Accessable,
};

use super::Exec;

#[derive(Debug, Args)]
pub struct Status {}

impl Exec for Status {
    fn exec(&self) -> anyhow::Result<()> {
        let repo = Repository::load()?;
        let branch = repo.head().branch().load()?;

        println!("On branch {}", branch.name);

        let mut staging_change = if let Some(sha1) = &branch.head {
            let stage = repo.stage()?;
            let head_commit = repo
                .wrap(Object::accessor(sha1))
                .load()?
                .map(|c| c.cast_commit());
            let head_tree = repo
                .wrap(Object::accessor(&head_commit.tree))
                .load()?
                .map(|t| t.cast_tree());
            compare_trees(&head_tree, &stage.map(|s| s.0))?
        } else {
            Vec::new()
        };

        staging_change.sort_by(|a, b| a.line.name.cmp(&b.line.name));

        if staging_change.is_empty().not() {
            println!(
                "
Changes to be committed:
  (use \"git restore --staged <file>...\" to unstage)"
            );
            for diff in &staging_change {
                match diff.kind {
                    ComparedKind::Modified => {
                        println!(
                            "{}",
                            format!("        modified:   {}", diff.line.name).yellow()
                        );
                    }
                    ComparedKind::Deleted => {
                        println!(
                            "{}",
                            format!("        deleted:    {}", diff.line.name).red()
                        );
                    }
                    ComparedKind::Added => {
                        println!(
                            "{}",
                            format!("        new file:   {}", diff.line.name).green()
                        );
                    }
                }
            }
        }

        let stage = repo.stage()?;
        let working_tree = repo.working_tree()?;
        let mut working_change = compare_trees(&stage.map(|s| s.0), &working_tree)?;

        working_change.sort_by(|a, b| a.line.name.cmp(&b.line.name));

        let changes_not_staged_for_commit = working_change
            .iter()
            .filter(|x| x.kind != ComparedKind::Added)
            .collect::<Vec<_>>();

        if changes_not_staged_for_commit.is_empty().not() {
            println!(
                "
Changes not staged for commit:
  (use \"git add <file>...\" to update what will be committed)
  (use \"git restore <file>...\" to discard changes in working directory)"
            );
            for diff in changes_not_staged_for_commit {
                match diff.kind {
                    ComparedKind::Modified => {
                        println!(
                            "{}",
                            format!("        modified:   {}", diff.line.name).yellow()
                        );
                    }
                    ComparedKind::Deleted => {
                        println!(
                            "{}",
                            format!("        deleted:    {}", diff.line.name).red()
                        );
                    }
                    ComparedKind::Added => {
                        println!(
                            "{}",
                            format!("        new file:   {}", diff.line.name).green()
                        );
                    }
                }
            }
        }

        let untracked = working_change
            .iter()
            .filter(|x| x.kind == ComparedKind::Added)
            .collect::<Vec<_>>();

        if untracked.is_empty().not() {
            println!(
                "
Untracked files:
  (use \"git add <file>...\" to include in what will be committed)"
            );
            for diff in untracked {
                println!("        {}", diff.line.name.green());
            }
        }

        if working_change.is_empty() && staging_change.is_empty() {
            println!("nothing to commit, working tree clean");
        }

        Ok(())
    }
}
