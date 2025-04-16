use super::mut_tree::MutableTree;
use crate::models::{
    repo::{Repository, WithRepo},
    tree::Tree,
};
use std::{collections::HashMap, io};

impl Repository {
    /// get the working directory of the repository
    pub fn working_tree(&self) -> io::Result<WithRepo<Tree>> {
        let mut working_tree = self.wrap(MutableTree {
            data: HashMap::new(),
            save_object: true,
        });

        working_tree.add_path(self.working_dir())?;

        Ok(working_tree.freeze())
    }
}
