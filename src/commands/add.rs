use std::env;

use clap::Args;
use colored::Colorize;

use crate::{
    console_output,
    models::{repo::Repository, stage::Stage},
    services::stage::StageService,
};

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
            console_output!("Nothing specified, nothing added.");
            console_output!("{}", "hint: Maybe you wanted to say 'git add .'?".yellow());
            return Ok(());
        }

        let mut stage = repo.stage()?.into_muter();

        for path in &self.paths {
            let path = env::current_dir()?.join(path);
            stage.add_path(&path)?;
        }

        stage.freeze().map(Stage).save()?;

        Ok(())
    }
}
