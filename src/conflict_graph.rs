use std::collections::{HashMap, HashSet};

pub struct ConflictGraph<T> {
    pub adjacency: HashMap<T, HashSet<T>>,
}

impl<T> ConflictGraph<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    pub fn new() -> Self {
        ConflictGraph {
            adjacency: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: T) {
        self.adjacency.entry(node).or_default();
    }
    pub fn add_conflict(&mut self, a: T, b: T) {
        if a == b {
            return;
        }
        self.adjacency
            .entry(b.clone())
            .or_default()
            .insert(a.clone());
        self.adjacency.entry(a).or_default().insert(b);
    }

    pub fn neighbors(&self, node: &T) -> impl Iterator<Item = &T> {
        self.adjacency.get(node).into_iter().flatten()
    }

    pub fn degree(&self, node: &T) -> usize {
        self.adjacency.get(node).map_or(0, |s| s.len())
    }
}
