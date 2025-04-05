use clap::Args;

use super::Exec;

#[derive(Debug, Args)]
pub struct Add {
    files: Vec<String>,
}

impl Exec for Add {
    fn exec(&self) -> anyhow::Result<()> {
        panic!("add is not implemented")
    }
}
