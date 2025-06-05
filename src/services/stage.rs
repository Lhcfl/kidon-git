use crate::models::{repo::WithRepo, stage::Stage};

use super::mut_tree::MutableTree;

pub trait StageService<'a> {
    fn into_muter(self) -> WithRepo<'a, MutableTree>;
}

impl<'a> StageService<'a> for WithRepo<'a, Stage> {
    fn into_muter(self) -> WithRepo<'a, MutableTree> {
        WithRepo::new(
            self.repo,
            MutableTree {
                data: self.unwrap().0.into_map(),
                save_object: true,
            },
        )
    }
}
