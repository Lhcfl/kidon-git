use crate::traits::DirContainer;

#[derive(Debug)]
pub enum RefsKind {
    Branch,
    Tag,
}

/// Referance to a commit
/// It may have a context of remote, but we don't store it here
#[derive(Debug)]
pub struct Refs {
    pub kind: RefsKind,
    pub sha1: String,
}

impl DirContainer for Refs {
    const DIRECTORY: &'static str = "refs";

    fn make_dir(root: &std::path::Path) -> std::io::Result<()> {
        let path = root.join(Self::DIRECTORY);
        std::fs::create_dir_all(&path)?;
        std::fs::create_dir_all(path.join("heads"))?;
        Ok(())
    }
}
