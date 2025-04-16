//! Tree Services

use super::stage::MutableTree;
use crate::{
    models::{
        object::Object,
        repo::{Repository, WithRepo},
        tree::{Tree, TreeLine, TreeLineKind},
    },
    traits::Accessable,
};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    io,
    path::Path,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparedKind {
    Added,
    Deleted,
    Modified,
}

pub struct ComparedLine {
    pub kind: ComparedKind,
    pub line: TreeLine,
}

impl Display for ComparedLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ComparedKind::Modified => {
                write!(f, "        modified:   {}", self.line.name)
            }
            ComparedKind::Deleted => {
                write!(f, "        deleted:    {}", self.line.name)
            }
            ComparedKind::Added => {
                write!(f, "        new file:   {}", self.line.name)
            }
        }
    }
}

/// 比较两个 tree
pub fn compare_trees(from: &WithRepo<Tree>, to: &WithRepo<Tree>) -> io::Result<Vec<ComparedLine>> {
    let from_map = from.get_map();
    let to_map = to.get_map();
    let all_items = from_map
        .keys()
        .chain(to_map.keys())
        .collect::<HashSet<&String>>();
    let mut res = Vec::new();

    for item in all_items.into_iter() {
        let item_from = from_map.get(item).copied();
        let item_to = to_map.get(item).copied();
        match (item_from, item_to) {
            (Some(item_from), Some(item_to)) if item_from.sha1 != item_to.sha1 => {
                if item_from.kind == TreeLineKind::Tree && item_to.kind == TreeLineKind::Tree {
                    let a = from
                        .wrap(Object::accessor(&item_from.sha1))
                        .load()?
                        .map(|a| a.cast_tree());
                    let b = to
                        .wrap(Object::accessor(&item_to.sha1))
                        .load()?
                        .map(|b| b.cast_tree());
                    res.append(&mut (compare_trees(&a, &b)?));
                } else {
                    res.push(ComparedLine {
                        kind: ComparedKind::Modified,
                        line: item_to.clone(),
                    });
                }
            }
            (Some(removed), None) => {
                res.push(ComparedLine {
                    kind: ComparedKind::Deleted,
                    line: removed.clone(),
                });
            }
            (None, Some(added)) => {
                res.push(ComparedLine {
                    kind: ComparedKind::Added,
                    line: added.clone(),
                });
            }
            _ => {}
        }
    }

    Ok(res)
}

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
