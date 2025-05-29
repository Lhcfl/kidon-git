use super::Exec;
use crate::{
    console_output, models::{object::Object, repo::Repository, Accessible}
};
use clap::Args;

#[derive(Debug, Args)]
pub struct CatFile {
    /// the sha1 of object
    sha1: String,
}

impl Exec for CatFile {
    fn exec(&self) -> anyhow::Result<()> {
        let repo = Repository::load()?;
        match self.sha1.as_str() {
            "index" => {
                let stage = repo.stage()?;
                console_output!("{stage}")
            }
            "working-tree" => {
                let working_tree = repo.working_tree()?;
                console_output!("{working_tree}")
            }
            _ => {
                let object = repo.wrap(Object::accessor(&self.sha1)).load()?;
                console_output!("{object}");
            }
        }
        Ok(())
    }
}
