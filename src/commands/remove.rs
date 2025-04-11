//! Remove files from stage area

use clap::Args;

use super::Exec;

#[derive(Debug, Args)]
pub struct Remove {
    files: Vec<String>,
}

impl Exec for Remove {
    fn exec(&self) -> anyhow::Result<()> {
        // TODO @leonard 你来写 rm
        // rm不需要真的删除文件，只需要删掉stage area的索引就行了
        panic!("rm is not implemented")
    }
}
