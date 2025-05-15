use crate::{
    models::repo::Repository, oj_output, services::commit::{CommitCreateResult, CommitService}
};
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

        let Some(message) = &self.message else {
            anyhow::bail!("commit message is required");
        };

        let res = repo.create_commit(message)?;

        match res {
            CommitCreateResult::Success(res) => {
                let tip = if res.branch.head.is_none() {
                    " (root-commit) "
                } else {
                    " "
                };

                println!(
                    "[{}{tip}{}] {message}",
                    res.branch.name,
                    &res.commit_sha1[0..7]
                );

                // Log commit status
                if let Some(compared) = res.compared {
                    println!("{} file changed", compared.len());
                    for line in compared {
                        println!("{line}");
                    }
                }

                oj_output!("{}", res.commit_sha1);
            }
            CommitCreateResult::NothingToCommit => {
                println!("nothing to commit, working tree clean");
            }
        }

        Ok(())
    }
}
