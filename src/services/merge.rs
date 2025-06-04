use std::collections::{HashMap, VecDeque};

use crate::models::commit::{Commit, CommitBuilder};
use crate::models::object::{Object, Sha1Able};
use crate::models::stage::{self, Stage};
use crate::models::{Accessible, branch::Branch, repo::Repository};
use crate::services::dump_tree::DumpTreeService;
use crate::oj_output;
use crate::services::tree::auto_merge_trees;

pub trait MergeService {
    fn merge(&self, theirs_branch: Branch) -> anyhow::Result<()>;
    fn get_merge_base(&self, commit1: &Commit, commit2: &Commit) -> anyhow::Result<Commit>;
}

impl MergeService for Repository {
    /// Merge another branch into the current branch.
    ///
    /// This method will merge the specified branch into the current branch.
    /// It will handle conflicts and return an error if the merge fails.

    fn merge(&self, theirs_branch: Branch) -> anyhow::Result<()> {
        let theirs_branch = self.wrap(theirs_branch);
        let ours_branch = self.head().load_branch()?;
        let ours_commit = ours_branch.get_current_commit()?;
        let theirs_commit = theirs_branch.get_current_commit()?;
        let base_commit = self.get_merge_base(&ours_commit, &theirs_commit)?;

        let base_tree = self
            .wrap(Object::accessor(&base_commit.tree))
            .load()?
            .map(|t| t.cast_tree());
        let ours_tree = self
            .wrap(Object::accessor(&ours_commit.tree))
            .load()?
            .map(|t| t.cast_tree());
        let theirs_tree = self
            .wrap(Object::accessor(&theirs_commit.tree))
            .load()?
            .map(|t| t.cast_tree());

        let (merged_tree, conflicts) = auto_merge_trees(&base_tree, &ours_tree, &theirs_tree)?;

        // ⚠️ 有冲突，输出冲突提示
        if !conflicts.is_empty() {
            for conflict in conflicts {
                if conflict.line_start == conflict.line_end {
                    oj_output!(
                        "Merge conflict in {}: {}",
                        conflict.file, conflict.line_start
                    );
                } else {
                    oj_output!(
                        "Merge conflict in {}: [{}, {}]",
                        conflict.file, conflict.line_start, conflict.line_end
                    );
                }
            }
            anyhow::bail!("Merge conflicts detected");
        }

        // ✅ 无冲突，生成合并提交
        let merged_tree_obj = self.wrap(Object::Tree(merged_tree));
        merged_tree_obj.save()?;

        let tree_sha1 = merged_tree_obj.sha1();

        let message = format!("Merge branch '{}'", theirs_branch.name);

        let merge_commit = Commit::new(CommitBuilder {
            tree: tree_sha1.into(),
            parent: Some(ours_commit.sha1().into()),
            message,
        });

        let sha1 = merge_commit.sha1();
        self.wrap(Object::Commit(merge_commit)).save()?;

        let merged_tree = merged_tree_obj.unwrap().cast_tree();
        let tree = self.wrap(Stage(merged_tree));
        tree.save()?;        
        self.dump_tree(tree.unwrap().0)?;

        let mut ours_branch_cloned = ours_branch.cloned();
        ours_branch_cloned.head = sha1.clone().into();
        ours_branch_cloned.save()?;

        // oj_output!("Merge successful: new HEAD is {}", sha1);
        Ok(())
    }

    fn get_merge_base(&self, commit1: &Commit, commit2: &Commit) -> anyhow::Result<Commit> {
        let mut visited = HashMap::new(); // 1 表示从 commit1 来，2 表示从 commit2 来，3 表示都到过
        let mut queue: VecDeque<(u8, crate::models::object::ObjectSha1)> = VecDeque::new();

        queue.push_back((1u8, commit1.sha1().into()));
        queue.push_back((2u8, commit2.sha1().into()));

        while let Some((source, sha)) = queue.pop_front() {
            // 如果访问过就合并标记
            let state = visited.entry(sha.clone()).or_insert(0);
            *state |= source;

            if *state == 3 {
                // 从两个方向都访问到了，找到 LCA
                let obj = self.wrap(Object::accessor(&sha)).load()?.unwrap();
                if let Object::Commit(cmt) = obj {
                    return Ok(cmt);
                } else {
                    anyhow::bail!("Object {} is not a commit", sha);
                }
            }

            // 向上继续遍历父节点
            let Object::Commit(commit) = self.wrap(Object::accessor(&sha)).load()?.unwrap() else {
                anyhow::bail!("Object {} is not a commit", sha);
            };

            if let Some(parent_sha) = commit.parent {
                queue.push_back((source, parent_sha));
            }
        }

        anyhow::bail!("No common ancestor found between the two commits")
    }
}
