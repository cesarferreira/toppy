use std::collections::{HashMap, HashSet};

use crate::process::model::ProcessRow;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub index: usize,
    pub children: Vec<u32>,
}

pub struct ProcessTree {
    pub roots: Vec<u32>,
    pub by_pid: HashMap<u32, TreeNode>,
}

impl ProcessTree {
    pub fn build(processes: &[ProcessRow]) -> Self {
        let mut children_map: HashMap<u32, Vec<u32>> =
            HashMap::with_capacity(processes.len() / 2);
        let mut index_by_pid: HashMap<u32, usize> = HashMap::with_capacity(processes.len());

        for (i, p) in processes.iter().enumerate() {
            index_by_pid.insert(p.pid, i);
            if let Some(parent) = p.parent_pid {
                children_map.entry(parent).or_default().push(p.pid);
            }
        }

        for children in children_map.values_mut() {
            children.sort_unstable();
        }

        let mut roots: Vec<u32> = processes
            .iter()
            .filter(|p| {
                p.parent_pid
                    .is_none_or(|parent| !index_by_pid.contains_key(&parent))
            })
            .map(|p| p.pid)
            .collect();
        roots.sort_unstable();

        let mut by_pid: HashMap<u32, TreeNode> = HashMap::with_capacity(processes.len());
        for (i, p) in processes.iter().enumerate() {
            let children = children_map.remove(&p.pid).unwrap_or_default();
            by_pid.insert(p.pid, TreeNode { index: i, children });
        }

        Self { roots, by_pid }
    }

    pub fn flatten_visible(
        &self,
        processes: &[ProcessRow],
        expanded: &HashSet<u32>,
    ) -> Vec<(u32, usize, bool)> {
        let mut out = Vec::new();
        for &root in &self.roots {
            self.walk(root, processes, expanded, 0, &mut out);
        }
        out
    }

    fn walk(
        &self,
        pid: u32,
        processes: &[ProcessRow],
        expanded: &HashSet<u32>,
        depth: usize,
        out: &mut Vec<(u32, usize, bool)>,
    ) {
        let Some(node) = self.by_pid.get(&pid) else {
            return;
        };
        let has_children = !node.children.is_empty();
        out.push((pid, depth, has_children));

        if has_children && expanded.contains(&pid) {
            for &child in &node.children {
                self.walk(child, processes, expanded, depth + 1, out);
            }
        }
    }
}
