use super::Exec;
use crate::{
    models::{object::Object, repo::Repository},
    models::Accessable,
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
                println!("{stage}")
            }
            "working-tree" => {
                let working_tree = repo.working_tree()?;
                println!("{working_tree}")
            }
            _ => {
                let object = repo.wrap(Object::accessor(&self.sha1)).load()?;
                println!("{object}");
            }
        }
        Ok(())
    }
}
