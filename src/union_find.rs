use std::collections::HashMap;
use std::hash::Hash;

struct NodeInfo<T: Hash + Eq + PartialEq + Clone> {
    pub parent: Option<T>,
    pub rank: usize,
}
impl<T: Hash + Eq + PartialEq + Clone> Default for NodeInfo<T> {
    fn default() -> Self {
        NodeInfo {
            parent: None,
            rank: 0
        }
    }
}

pub struct UnionFind<T: Hash + Eq + PartialEq + Clone> {
    nodes: HashMap<T, NodeInfo<T>>,
}
impl<T: Hash + Eq + PartialEq + Clone> Default for UnionFind<T> {
    fn default() -> Self {
        UnionFind {
            nodes: Default::default()
        }
    }
}
impl<T: Hash + Eq + PartialEq + Clone> UnionFind<T> {
    pub fn insert(&mut self, value: T) {
        self.nodes.entry(value).or_insert(NodeInfo::default());
    }

    pub fn union(&mut self, x: &T, y: &T) {
        self.insert(x.clone());
        self.insert(y.clone());
        let (x_root, y_root) = (self.find(x).unwrap().clone(), self.find(y).unwrap().clone());

        if x_root != y_root {
            let x_rank = self.nodes.get(&x_root).unwrap().rank;
            let y_rank = self.nodes.get(&y_root).unwrap().rank;
            if x_rank < y_rank {
                self.nodes.get_mut(&y_root).unwrap().parent = Some(x_root)
            } else {
                self.nodes.get_mut(&y_root).unwrap().parent = Some(x_root.clone());
                if x_rank == y_rank {
                    self.nodes.get_mut(&x_root).unwrap().rank += 1;
                }
            }
        }
    }

    pub fn find<'a>(&'a self, mut value: &'a T) -> Option<&T> {
        if !self.nodes.contains_key(value) {
            return None
        }
        while let Some(NodeInfo { parent: Some(parent), .. }) = self.nodes.get(value) {
            value = parent
        }
        Some(value)
    }
}