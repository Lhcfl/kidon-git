use crate::traits::DirContainer;

pub enum RefsKind {
    Branch,
    Tag,
}

/// Referance to a commit
/// It may have a context of remote, but we don't store it here
pub struct Refs {
    pub kind: RefsKind,
    pub sha1: String,
}

impl DirContainer for Refs {
    const DIRECTORY: &'static str = "refs";
}
