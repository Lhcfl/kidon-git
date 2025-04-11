use std::{env, path::Path};

use clap::Args;
use colored::Colorize;

use crate::{models::repo::Repository, services::stage::StageService};

use super::Exec;

#[derive(Debug, Args)]
pub struct Add {
    /// the paths of files to add
    paths: Vec<String>,
}

impl Exec for Add {
    fn exec(&self) -> anyhow::Result<()> {
        let repo = Repository::load()?;

        if self.paths.is_empty() {
            println!("Nothing specified, nothing added.");
            println!("{}", "hint: Maybe you wanted to say 'git add .'?".yellow());
            return Ok(());
        }

        let mut stage = repo.stage()?;

        for path in &self.paths {
            let path = env::current_dir()?.join(path);
            stage.add_path(&path)?;
        }

        stage.save()?;

        Ok(())
    }
}
