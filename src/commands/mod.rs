//! Command interface

use clap::Subcommand;
use enum_dispatch::enum_dispatch;

mod add;
mod branch;
mod cat_file;
mod checkout;
mod commit;
mod fetch;
mod init;
mod log;
mod merge;
mod pull;
mod push;
mod remove;
mod status;

#[enum_dispatch]
pub trait Exec {
    /// execute the command
    fn exec(&self) -> anyhow::Result<()>;
}

#[derive(Debug, Subcommand)]
#[enum_dispatch(Exec)]
pub enum Commands {
    /// Record changes to the repository
    Commit(commit::Commit),
    /// Add file contents to the index
    Add(add::Add),
    /// Create an empty Git repository or reinitialize an existing one
    Init(init::Init),
    /// List, create, or delete branches
    Branch(branch::Branch),
    /// Switch branches or restore working tree files
    Checkout(checkout::Checkout),
    /// Join two or more development histories together
    Merge(merge::Merge),
    /// Remove files from the working tree and from the index
    #[command(aliases(["remove"]))]
    Rm(remove::Remove),
    /// Show commit logs
    Log(log::Log),
    /// Show the working tree status
    Status(status::Status),
    /// Download objects and refs from another repository
    Fetch(fetch::Fetch),
    /// Fetch from and integrate with another repository or a local branch
    Pull(pull::Pull),
    /// Update remote refs along with associated objects
    Push(push::Push),
    /// (For debug) Display information about a object
    CatFile(cat_file::CatFile),
}
