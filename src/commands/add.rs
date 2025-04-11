use clap::Args;
use colored::Colorize;

use super::Exec;

#[derive(Debug, Args)]
pub struct Add {
    /// the paths of files to add
    paths: Vec<String>,
}

impl Exec for Add {
    fn exec(&self) -> anyhow::Result<()> {
        if self.paths.len() == 0 {
            println!("Nothing specified, nothing added.");
            println!("{}", "hint: Maybe you wanted to say 'git add .'?".yellow());
            return Ok(());
        }

        panic!("add is not implemented")
    }
}
