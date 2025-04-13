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
    fn exec(&self) -> anyhow::Result<()> {
        // TODO @leonard 请（）
        // 自行解决如何把指定文件夹 dump 到一个 tree（）
        panic!("commit is not implemented")
    }
}
