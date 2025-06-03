//! Tree Services

use crate::{
    models::{
        object::Object,
        repo::WithRepo,
        tree::{Tree, TreeLine, TreeLineKind},
    },
    models::Accessible,
};
use std::{
    collections::HashSet,
    fmt::Display,
    io,
    path::Path,
};
use std::collections::HashMap;

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

impl ComparedLine {
    fn prepent_parent(mut self, path: &Path) -> Self {
        self.line.name = path.join(&self.line.name).to_string_lossy().to_string();
        self
    }
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

fn compare_tree_with_path(
    root: &Path,
    from: &WithRepo<Tree>,
    to: &WithRepo<Tree>,
) -> io::Result<Vec<ComparedLine>> {
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
                    res.append(&mut compare_tree_with_path(
                        &root.join(&item_from.name),
                        &a,
                        &b,
                    )?);
                } else {
                    res.push(
                        ComparedLine {
                            kind: ComparedKind::Modified,
                            line: item_to.clone(),
                        }
                        .prepent_parent(root),
                    );
                }
            }
            (Some(removed), None) => {
                res.push(
                    ComparedLine {
                        kind: ComparedKind::Deleted,
                        line: removed.clone(),
                    }
                    .prepent_parent(root),
                );
            }
            (None, Some(added)) => {
                res.push(
                    ComparedLine {
                        kind: ComparedKind::Added,
                        line: added.clone(),
                    }
                    .prepent_parent(root),
                );
            }
            _ => {}
        }
    }

    Ok(res)
}

/// 比较两个 tree
pub fn compare_trees(from: &WithRepo<Tree>, to: &WithRepo<Tree>) -> io::Result<Vec<ComparedLine>> {
    compare_tree_with_path(Path::new(""), from, to)
}

pub struct Conflict {
    pub file: String,
    pub line_start: usize,
    pub line_end: usize,
}

/// 自动合并 tree，如果冲突，则返回注入了冲突标记的 tree 和冲突信息
pub fn auto_merge_trees(
    base: &WithRepo<Tree>,
    ours: &WithRepo<Tree>,
    theirs: &WithRepo<Tree>,
) -> anyhow::Result<(Tree, Vec<Conflict>)> {
    let mut merged_map = HashMap::new();
    let mut conflicts = Vec::new();

    let base_map = base.get_map();
    let ours_map = ours.get_map();
    let theirs_map = theirs.get_map();

    let all_items: HashSet<_> = base_map
        .keys()
        .chain(ours_map.keys())
        .chain(theirs_map.keys())
        .collect();

    for item in all_items {
        let base_line = base_map.get(item).copied();
        let ours_line = ours_map.get(item).copied();
        let theirs_line = theirs_map.get(item).copied();

        match (base_line, ours_line, theirs_line) {
            (None, Some(o), None) | (None, None, Some(o))=> {
                // only ours added
                merged_map.insert(item.clone(), o.clone());
            }
            (_, Some(o), Some(t)) if o.sha1 == t.sha1 => {
                // o==t
                merged_map.insert(item.clone(), o.clone());
            }
            (Some(b), Some(o), Some(t)) if b.sha1 == o.sha1 => {
                // b==o!=t
                // only theirs modified 
                merged_map.insert(item.clone(), t.clone());
            }
            (Some(b), Some(o), Some(t)) if b.sha1 == t.sha1 => {
                // only ours modified
                merged_map.insert(item.clone(), o.clone());
            }
            (Some(b), Some(o), Some(t)) => {
                // ❌ conflict: both modified and different from base                // ⬇️ load blob content from o/t
                let o_blob = ours.wrap(Object::accessor(&o.sha1)).load()?.unwrap().cast_blob();
                let t_blob = theirs.wrap(Object::accessor(&t.sha1)).load()?.unwrap().cast_blob();

                let (merged, start, end) = inject_conflict_markers(&o_blob, &t_blob);

                let merged_blob = Blob::from_string(&merged);
                let blob_obj = Object::Blob(merged_blob.clone());
                blob_obj.save()?;
                let sha1 = blob_obj.sha1();

                merged_map.insert(
                    item.clone(),
                    TreeLine {
                        name: item.clone(),
                        kind: TreeLineKind::File,
                        sha1: sha1.into(),
                    },
                );

                conflicts.push(Conflict {
                    file: item.clone(),
                    line_start: start,
                    line_end: end,
                });
            }
            (_, Some(o), None) => {
                // only ours added
                merged_map.insert(item.clone(), o.clone());
            }
            (_, None, Some(t)) => {
                // only theirs added
                merged_map.insert(item.clone(), t.clone());
            }
            (_, None, None) => {} // both deleted
            _ => {}
        }
    }

    let merged_tree = Tree::from(merged_map);
    Ok((merged_tree, conflicts))
}

/// 注入 Git 样式冲突标记，返回合并内容和冲突行区间
fn inject_conflict_markers(ours: &Blob, theirs: &Blob) -> (String, usize, usize) {
    let ours_lines: Vec<&str> = ours.content.lines().collect();
    let theirs_lines: Vec<&str> = theirs.content.lines().collect();

    let start = 1;
    let end = std::cmp::max(ours_lines.len(), theirs_lines.len());

    let merged = format!(
        "<<<<<<< ours\n{}\n=======\n{}\n>>>>>>> theirs\n",
        ours.content.trim_end(),
        theirs.content.trim_end()
    );
    (merged, start, end)
}
