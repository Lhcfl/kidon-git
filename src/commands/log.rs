use super::Exec;
use crate::{
    console_output,
    models::repo::Repository,
    services::object::ObjectService,
};
use clap::Args;
use colored::Colorize;
use std::fmt::Debug;

#[derive(Debug, Args)]
pub struct Log {
    #[arg(default_value("10"))]
    number: u32,
}

impl Exec for Log {
    fn exec(&self) -> anyhow::Result<()> {
        let repo = Repository::load()?;
        let Ok(branch) = repo.head().load_branch() else {
            anyhow::bail!(
                "your current branch '{}' does not have any commits yet",
                repo.head().branch_name
            );
        };

        let mut next_sha1 = Some(branch.unwrap().head);

        for _ in 1..self.number {
            let Some(sha1) = next_sha1 else {
                return Ok(());
            };

            let commit = repo.load_object(&sha1)?.unwrap().cast_commit();

            console_output!("{} {}", "commit".yellow(), sha1);
            console_output!(
                "Date:   {}",
                commit.time().naive_local().format("%Y-%m-%d %H:%M:%S")
            );
            console_output!();
            commit
                .message
                .split('\n')
                .take(5)
                .for_each(|s| console_output!("    {s}"));
            console_output!();
            next_sha1 = commit.parent;
        }

        Ok(())
    }
}
