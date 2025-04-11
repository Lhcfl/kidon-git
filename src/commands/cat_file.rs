use super::Exec;
use crate::{
    models::{object::Object, repo::Repository},
    traits::Accessable,
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
        if self.sha1 == "index" {
            let stage = repo.stage()?;
            println!("{stage}")
        } else {
            let object = repo.wrap(Object::accessor(&self.sha1)).load()?;
            println!("{object}");
        }
        Ok(())
    }
}
