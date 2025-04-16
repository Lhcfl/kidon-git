//! Remove files from stage area
use std::env;

use clap::Args;

use crate::{
    models::{repo::Repository, stage::Stage},
    services::stage::StageService,
};
use super::Exec;

#[derive(Debug, Args)]
pub struct Remove {
    #[arg(short, long)]
    pub recursive: bool,
    
    path: Vec<String>,
}

impl Exec for Remove {
    fn exec(&self) -> anyhow::Result<()> {
        // rm不需要真的删除文件，只需要删掉stage area的索引就行了
        let repo = Repository::load()?;
        let mut stage = repo.stage()?.into_muter();
        
        for path in &self.path {
            let path = env::current_dir()?.join(path);
            if path.is_dir() && !self.recursive {
                return Err(anyhow::anyhow!("rm: {} is a directory", path.display()));
            }
        }
        for path in &self.path {
            let path = env::current_dir()?.join(path);
            stage.remove_path(&path)?;
        }
        stage.freeze().map(Stage).save()?;
        Ok(())
    }
}
