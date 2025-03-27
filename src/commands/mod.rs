use clap::Subcommand;

mod add;
mod branch;
mod checkout;
mod commit;
mod init;
mod merge;
mod remove;

#[derive(Debug, Subcommand)]
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
    /// Download objects and refs from another repository
    Fetch,
    /// Fetch from and integrate with another repository or a local branch
    Pull,
    /// Update remote refs along with associated objects
    Push,
}

pub trait Exec {
    /// execute the command
    fn exec(&self);
}

impl Commands {
    pub fn show(&self) {
        println!("{self:?}");
    }
}

impl Exec for Commands {
    fn exec(&self) {
        use Commands::*;
        match self {
            Commit(data) => data.exec(),
            Add(data) => data.exec(),
            Init(data) => data.exec(),
            Branch(data) => data.exec(),
            Checkout(data) => data.exec(),
            Merge(data) => data.exec(),
            Rm(data) => data.exec(),
            Fetch => {
                panic!("Not Implemented");
            }
            Pull => {
                panic!("Not Implemented");
            }
            Push => {
                panic!("Not Implemented");
            }
        }
    }
}
