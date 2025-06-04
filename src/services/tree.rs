//! Tree Services

use crate::models::object::Sha1Able;
use crate::{
    models::Accessible,
    models::{
        object::Object,
        repo::WithRepo,
        tree::{Tree, TreeLine, TreeLineKind},
    },
};
use std::collections::HashMap;
use std::io::ErrorKind::ConnectionAborted;
use std::{collections::HashSet, fmt::Display, io, path::Path};

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

impl Conflict {
    pub(crate) fn clone(&self) -> Conflict {
        Conflict {
            file: self.file.clone(),
            line_start: self.line_start,
            line_end: self.line_end,
        }
    }
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
        .collect::<HashSet<&String>>();

    for item in all_items.into_iter() {
        let base_line = base_map.get(item).copied();
        let ours_line = ours_map.get(item).copied();
        let theirs_line = theirs_map.get(item).copied();

        match (base_line, ours_line, theirs_line) {
            (_, None, None) => {
                // base deleted, both ours and theirs deleted
                // do nothing
            }

            (None, Some(i), None) | (None, None, Some(i)) => {
                // only ours added
                merged_map.insert(item.clone(), i.clone());
            }
            (Some(b), Some(i), None) | (Some(b), None, Some(i)) if b.sha1 == i.sha1 => {
                // only ours deleted, or only theirs deleted
                // do nothing
            }

            (_, Some(o), Some(t)) if o.sha1 == t.sha1 => {
                // o==t
                merged_map.insert(item.clone(), o.clone());
            }
            (Some(b), Some(f), Some(s)) | (Some(b), Some(s), Some(f)) if b.sha1 == f.sha1 => {
                // f unchanged, s modified
                merged_map.insert(item.clone(), s.clone());
            }

            (_, None, Some(_)) | (_, Some(_), None) => {
                panic!("Not required.")
            }

            (Some(b), Some(o), Some(t)) if o.sha1 != t.sha1 => {
                // Conflict: o!=t 
                if o.kind!= t.kind {
                    anyhow::bail!(
                        "Conflict: different kinds of objects, o: {}, t: {}",
                        o.kind,
                        t.kind
                    );
                }
                if o.kind == TreeLineKind::Tree {
                    // 处理子目录冲突
                    let b_tree = base.wrap(Object::accessor(&b.sha1)).load()?.unwrap().cast_tree();
                    let o_tree = ours.wrap(Object::accessor(&o.sha1)).load()?.unwrap().cast_tree();
                    let t_tree = theirs.wrap(Object::accessor(&t.sha1)).load()?.unwrap().cast_tree();
                    let (merged_subtree, sub_conflicts) =
                        auto_merge_trees(&base.wrap(Object::Tree(b_tree)), &ours.wrap(Object::Tree(o_tree)), &theirs.wrap(Object::Tree(t_tree)))?;
                    merged_map.insert(item.clone(), TreeLine {
                        name: item.clone(),
                        kind: TreeLineKind::Tree,
                        sha1: merged_subtree.sha1().into(),
                    });
                    conflicts.extend(sub_conflicts);
                    continue;
                }
                handle_conflict(&mut conflicts, o, t, &ours, &theirs)?;
            }
            (None, Some(o), Some(t)) if o.sha1 != t.sha1 => {
                // Conflict: o!=t 
                if o.kind!= t.kind {
                    anyhow::bail!("Conflict: different kinds of objects, o: {}, t: {}", o.kind, t.kind);
                }
                if o.kind == TreeLineKind::Tree {
                    // 处理子目录冲突
                    let o_tree = ours
                        .wrap(Object::accessor(&o.sha1))
                        .load()?
                        .unwrap()
                        .cast_tree();
                    let t_tree = theirs
                        .wrap(Object::accessor(&t.sha1))
                        .load()?
                        .unwrap()
                        .cast_tree();
                    // ... right?? @linca
                    let (merged_subtree, sub_conflicts) = auto_merge_trees(
                        &base.wrap(o_tree.clone()),
                        &ours.wrap(o_tree),
                        &theirs.wrap(t_tree),
                    )?;
                    merged_map.insert(
                        item.clone(),
                        TreeLine {
                            name: item.clone(),
                            kind: TreeLineKind::Tree,
                            sha1: merged_subtree.sha1().into(),
                        },
                    );
                    conflicts.extend(sub_conflicts);
                    continue;
                }
                handle_conflict(&mut conflicts, o, t, &ours, &theirs)?;
            }
            _ => {}
        }
    }

    let merged_tree = Tree::from(merged_map);
    Ok((merged_tree, conflicts))
}

fn handle_conflict(
    conflicts: &mut Vec<Conflict>,
    o: &TreeLine,
    t: &TreeLine,
    ours: &WithRepo<Tree>,
    theirs: &WithRepo<Tree>,
) -> anyhow::Result<()> {
    // 处理冲突，返回冲突信息
    match o.kind {
        TreeLineKind::File => {
            let a = ours
                .wrap(Object::accessor(&o.sha1))
                .load()?
                .map(|a| a.cast_blob());
            let a_str = a.as_string();
            let a_lines = a_str.lines();
            let b = theirs
                .wrap(Object::accessor(&t.sha1))
                .load()?
                .map(|b| b.cast_blob());
            let b_str = b.as_string();
            let b_lines = b_str.lines();
            let mut conflicting = false;
            let mut current_conflict = Conflict {
                file: o.name.clone(),
                line_start: 0,
                line_end: 0,
            };

            for (index, (a_line, b_line)) in a_lines.zip(b_lines).enumerate() {
                if a_line != b_line {
                    // has conflict
                    if !conflicting {
                        // starting conflict block
                        conflicting = true;
                        current_conflict.line_start = index + 1;
                    }
                    current_conflict.line_end = index + 1;
                } else {
                    // not conflicting
                    conflicting = false;
                    if current_conflict.line_start != 0 {
                        // add conflict block to conflicts
                        conflicts.push(current_conflict.clone());
                        current_conflict = Conflict {
                            file: o.name.clone(),
                            line_start: 0,
                            line_end: 0,
                        };
                    }
                }
            }
            Ok(())
        }
        TreeLineKind::Executable => {
            panic!("Not required.")
        }
        TreeLineKind::Symlink => {
            panic!("Not required.")
        }
        TreeLineKind::Tree => {
            panic!("Shouldn't go here.")
        }
    }
}
// /// 注入 Git 样式冲突标记，返回合并内容和冲突行区间
// fn inject_conflict_markers(ours: &Blob, theirs: &Blob) -> (String, usize, usize) {
//     let ours_lines: Vec<&str> = ours.content.lines().collect();
//     let theirs_lines: Vec<&str> = theirs.content.lines().collect();
//
//     let start = 1;
//     let end = std::cmp::max(ours_lines.len(), theirs_lines.len());
//
//     let merged = format!(
//         "<<<<<<< ours\n{}\n=======\n{}\n>>>>>>> theirs\n",
//         ours.content.trim_end(),
//         theirs.content.trim_end()
//     );
//     (merged, start, end)
// }
