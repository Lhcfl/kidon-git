use clap::Args;

use super::Exec;

#[derive(Debug, Args)]
pub struct Rm {
    files: Vec<String>,
}

impl Exec for Rm {
    fn exec(&self) {
        panic!("rm is not implemented")
    }
}
