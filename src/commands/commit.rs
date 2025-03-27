use clap::Args;

use super::Exec;

#[derive(Debug, Args)]
pub struct Commit {
    /// commit message
    #[arg(short, long)]
    pub message: Option<String>,

    /// auto add
    #[arg(short, long)]
    pub add: bool,
}

impl Exec for Commit {
    fn exec(&self) {
        panic!("commit is not implemented")
    }
}
