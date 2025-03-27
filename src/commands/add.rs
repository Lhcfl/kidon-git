use clap::Args;

use super::Exec;

#[derive(Debug, Args)]
pub struct Add {
    files: Vec<String>,
}

impl Exec for Add {
    fn exec(&self) {
        panic!("add is not implemented")
    }
}
