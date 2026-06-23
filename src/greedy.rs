use crate::container::Container;
use crate::decoder::Solution;
use crate::instance::Instance;

/// First-fit-decreasing greedy baseline: items are sorted by descending area, then
/// placed via first-fit across the open bins using the conflict-aware, auto-rotating
/// `Container::try_insert` (a new bin is opened when none accepts the item). Returns
/// a `decoder::Solution` so it shares the serializer with the BRKGA decoder.
pub fn solve(instance: &Instance) -> Solution {
    let n = instance.items.len();
    let mut order: Vec<usize> = (0..n).collect();
    let area = |i: usize| instance.items[i].width * instance.items[i].height;
    order.sort_by(|&a, &b| area(b).cmp(&area(a))); // descending area

    let mut bins: Vec<Container> = Vec::new();
    let mut unplaced: Vec<usize> = Vec::new();

    for idx in order {
        let item = &instance.items[idx];
        let mut placed = bins
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
}
