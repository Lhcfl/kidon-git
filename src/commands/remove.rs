use clap::Args;

use super::Exec;

#[derive(Debug, Args)]
pub struct Remove {
    files: Vec<String>,
}

impl Exec for Remove {
    fn exec(&self) -> anyhow::Result<()> {
        panic!("rm is not implemented")
    }
}
