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
            rank: 0,
        }
    }
}

pub struct UnionFind<T: Hash + Eq + PartialEq + Clone> {
    nodes: HashMap<T, NodeInfo<T>>,
}
impl<T: Hash + Eq + PartialEq + Clone> Default for UnionFind<T> {
    fn default() -> Self {
        UnionFind {
            nodes: Default::default(),
        }
    }
}
impl<T: Hash + Eq + PartialEq + Clone> UnionFind<T> {
    pub fn insert(&mut self, value: T) {
        self.nodes.entry(value).or_insert(NodeInfo::default());
    }

    pub fn union(&mut self, x: T, y: T) {
        let (x_root, y_root) = (self.find_equivalence_mut(x), self.find_equivalence_mut(y));

        if x_root != y_root {
            let x_rank = self.nodes.get(&x_root).unwrap().rank;
            let y_rank = self.nodes.get(&y_root).unwrap().rank;
            if x_rank > y_rank {
                self.nodes.get_mut(&y_root).unwrap().parent = Some(x_root)
            } else {
                self.nodes.get_mut(&x_root).unwrap().parent = Some(y_root.clone());
                if x_rank == y_rank {
                    self.nodes.get_mut(&y_root).unwrap().rank += 1;
                }
            }
        }
    }

    pub fn find_equivalence_mut(&mut self, mut value: T) -> T {
        // We insert the value if it does not already exist
        if !self.nodes.contains_key(&value) {
            self.insert(value.clone());
            return value.clone();
        }

        let mut path = vec![value.clone()];
        while let Some(NodeInfo {
            parent: Some(parent),
            ..
        }) = self.nodes.get(&value)
        {
            value = parent.clone();
            path.push(value.clone());
        }

        // Now that we found the representative of the equivalence class,
        // we compress the paths from the children to said representative
        for n in path {
            self.nodes.get_mut(&n).unwrap().parent = Some(value.clone())
        }
        value.clone()
    }

    pub fn find_equivalence(&self, mut value: T) -> Option<T> {
        if !self.nodes.contains_key(&value) {
            return None;
        }

        while let Some(NodeInfo {
            parent: Some(parent),
            ..
        }) = self.nodes.get(&value)
        {
            if parent == &value {
                break;
            }
            value = parent.clone();
        }
        Some(value.clone())
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.nodes.keys()
    }
}
