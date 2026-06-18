use std::collections::{HashMap, HashSet};

use crate::conflict_graph::ConflictGraph;

fn greedy_coloring<T>(conflict_graph: ConflictGraph<T>) -> HashMap<T, usize>
where
    T: Eq + std::hash::Hash + Clone,
{
    let mut colors: HashMap<T, usize> = HashMap::new();
    for node in conflict_graph.adjacency.keys() {
        let used: HashSet<usize> = conflict_graph
            .neighbors(node)
            .filter_map(|n| colors.get(n).copied())
            .collect();
        let color = (0..).find(|c| !used.contains(c)).unwrap();
        colors.insert(node.clone(), color);
    }

    colors
}
