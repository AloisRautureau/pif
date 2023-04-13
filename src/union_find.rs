use std::collections::HashMap;
use std::hash::Hash;

struct NodeInfo<T: Hash + Eq + PartialEq + Clone> {
    pub _parent: Option<T>,
    pub _rank: usize,
}
impl<T: Hash + Eq + PartialEq + Clone> Default for NodeInfo<T> {
    fn default() -> Self {
        NodeInfo {
            _parent: None,
            _rank: 0,
        }
    }
}

pub struct UnionFind<T: Hash + Eq + PartialEq + Clone> {
    _nodes: HashMap<T, NodeInfo<T>>,
}
impl<T: Hash + Eq + PartialEq + Clone> Default for UnionFind<T> {
    fn default() -> Self {
        UnionFind {
            _nodes: Default::default(),
        }
    }
}
impl<T: Hash + Eq + PartialEq + Clone> UnionFind<T> {
    pub fn _insert(&mut self, value: T) {
        self._nodes.entry(value).or_insert(NodeInfo::default());
    }

    pub fn _union(&mut self, x: &T, y: &T) {
        self._insert(x.clone());
        self._insert(y.clone());
        let (x_root, y_root) = (
            self._find(x).unwrap().clone(),
            self._find(y).unwrap().clone(),
        );

        if x_root != y_root {
            let x_rank = self._nodes.get(&x_root).unwrap()._rank;
            let y_rank = self._nodes.get(&y_root).unwrap()._rank;
            if x_rank < y_rank {
                self._nodes.get_mut(&y_root).unwrap()._parent = Some(x_root)
            } else {
                self._nodes.get_mut(&y_root).unwrap()._parent = Some(x_root.clone());
                if x_rank == y_rank {
                    self._nodes.get_mut(&x_root).unwrap()._rank += 1;
                }
            }
        }
    }

    pub fn _find<'a>(&'a self, mut value: &'a T) -> Option<&T> {
        if !self._nodes.contains_key(value) {
            return None;
        }
        while let Some(NodeInfo {
            _parent: Some(parent),
            ..
        }) = self._nodes.get(value)
        {
            value = parent
        }
        Some(value)
    }
}
