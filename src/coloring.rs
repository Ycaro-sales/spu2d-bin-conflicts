use std::collections::{HashMap, HashSet};

use crate::conflict_graph::ConflictGraph;

/// Greedily color the conflict graph: assign each node the smallest color not
/// used by an already-colored neighbor, so conflicting (adjacent) nodes always
/// receive different colors and every color class is a conflict-free set.
///
/// Nodes are processed in a deterministic Welsh-Powell order (descending degree,
/// ties broken by node value), which tends to use fewer colors and makes the
/// result reproducible despite the underlying `HashMap` storage.
pub fn greedy_coloring<T>(graph: &ConflictGraph<T>) -> HashMap<T, usize>
where
    T: Eq + std::hash::Hash + Clone + Ord,
{
    let mut order: Vec<&T> = graph.adjacency.keys().collect();
    order.sort_by(|a, b| {
        (std::cmp::Reverse(graph.degree(a)), *a).cmp(&(std::cmp::Reverse(graph.degree(b)), *b))
    });

    let mut colors: HashMap<T, usize> = HashMap::new();
    for node in order {
        let used: HashSet<usize> = graph
            .neighbors(node)
            .filter_map(|n| colors.get(n).copied())
            .collect();
        let color = (0..).find(|c| !used.contains(c)).unwrap();
        colors.insert(node.clone(), color);
    }

    colors
}
