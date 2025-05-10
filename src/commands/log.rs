use super::Exec;
use crate::{
    models::{object::Object, repo::Repository},
    traits::Accessable,
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
        let mut next_sha1 = repo.head().branch().load()?.head.clone();

        if next_sha1.is_none() {
            println!("It is a new repository, no commit yet");
            return Ok(());
        }

        for _ in 1..self.number {
            let Some(sha1) = next_sha1 else {
                return Ok(());
            };

            let obj = repo.wrap(Object::accessor(&sha1)).load()?.unwrap();
            let Object::Commit(commit) = obj else {
                anyhow::bail!(
                    "bad object type: object {sha1} is not a commit, but a {}",
                    obj.object_type()
                );
            };

            println!("{} {}", "commit".yellow(), sha1);
            println!(
                "Date:   {}",
                commit.time().naive_local().format("%Y-%m-%d %H:%M:%S")
            );
            println!();
            commit
                .message
                .split('\n')
                .take(5)
                .for_each(|s| println!("    {}", s));
            println!();
            next_sha1 = commit.parent;
        }

        Ok(())
    }
}
