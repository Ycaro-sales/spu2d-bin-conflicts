use std::collections::BTreeMap;

use crate::coloring::greedy_coloring;
use crate::container::Container;
use crate::decoder::Solution;
use crate::instance::Instance;

/// Coloring-based first-fit-decreasing greedy.
///
/// The conflict graph is greedily colored so that each color class is a
/// conflict-free set, then items are packed **per color**: every color gets its
/// own dedicated bins (sorted by descending area, first-fit, opening another bin
/// for the color when its items overflow). Bins never mix colors, so conflicts
/// are impossible by construction rather than gated per insert. Returns a
/// `decoder::Solution` so it shares the serializer with the BRKGA decoder.
pub fn solve(instance: &Instance) -> Solution {
    let n = instance.items.len();
    let area = |i: usize| instance.items[i].width * instance.items[i].height;

    // Group item indices by their color (BTreeMap -> deterministic color order).
    let colors = greedy_coloring(&instance.conflicts);
    let mut by_color: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for idx in 0..n {
        let c = colors.get(&idx).copied().unwrap_or(0);
        by_color.entry(c).or_default().push(idx);
    }

    let mut bins: Vec<Container> = Vec::new();
    let mut unplaced: Vec<usize> = Vec::new();

    for (_color, mut group) in by_color {
        group.sort_by(|&a, &b| area(b).cmp(&area(a))); // descending area
        // Only first-fit into bins opened for this color, keeping bins
        // color-homogeneous.
        let start = bins.len();
        for idx in group {
            let item = &instance.items[idx];
            let mut placed = bins[start..]
                .iter_mut()
                .any(|b| b.try_insert(item, idx, &instance.conflicts).is_some());
            if !placed {
                let mut bin = Container::new(instance.width, instance.height);
                if bin.try_insert(item, idx, &instance.conflicts).is_some() {
                    bins.push(bin);
                    placed = true;
                }
            }
            if !placed {
                unplaced.push(idx);
            }
        }
    }

    Solution { bins, unplaced }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conflict_graph::ConflictGraph;
    use crate::item::Item;

    fn item(width: u32, height: u32) -> Item {
        Item {
            height,
            width,
            client: 0,
        }
    }

    fn instance(width: u32, height: u32, items: Vec<Item>, edges: &[(usize, usize)]) -> Instance {
        let mut conflicts = ConflictGraph::new();
        for i in 0..items.len() {
            conflicts.add_node(i);
        }
        for &(a, b) in edges {
            conflicts.add_conflict(a, b);
        }
        Instance {
            items,
            width,
            height,
            conflicts,
        }
    }

    #[test]
    fn packs_non_conflicting_items_into_one_bin() {
        let inst = instance(10, 10, vec![item(2, 2), item(3, 3), item(4, 4)], &[]);
        let sol = solve(&inst);
        assert_eq!(sol.bins_used(), 1);
        assert!(sol.unplaced.is_empty());
    }

    #[test]
    fn conflicts_force_separate_bins() {
        let inst = instance(10, 10, vec![item(2, 2), item(2, 2)], &[(0, 1)]);
        let sol = solve(&inst);
        assert_eq!(sol.bins_used(), 2);
    }

    #[test]
    fn same_color_items_share_a_bin_across_a_hub_conflict() {
        // Item 0 conflicts with both 1 and 2, but 1 and 2 do not conflict.
        // Coloring: 0 -> color 0; 1 and 2 share color 1 (an independent set).
        // So 0 lands in its own color-0 bin, while 1 and 2 (same color, both
        // small) pack together into a single color-1 bin: 2 bins total.
        let inst = instance(10, 10, vec![item(2, 2), item(2, 2), item(2, 2)], &[(0, 1), (0, 2)]);
        let sol = solve(&inst);
        assert_eq!(sol.bins_used(), 2);
        assert!(sol.unplaced.is_empty());
        // The two same-color items 1 and 2 are in the same bin.
        let bin_of = |idx: usize| {
            sol.bins
                .iter()
                .position(|b| b.placements().contains_key(&idx))
                .expect("item placed")
        };
        assert_eq!(bin_of(1), bin_of(2));
        assert_ne!(bin_of(0), bin_of(1));
    }
}
