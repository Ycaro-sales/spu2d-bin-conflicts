use std::sync::Arc;

use genetic_algorithm::strategy::evolve::prelude::*;

use crate::container::Container;
use crate::encoder::{Decoded, build_genotype, interpret};
use crate::instance::Instance;

/// A decoded packing: the bins (each a `Container`) and the items that could not
/// be placed in their forced orientation even in a fresh bin.
pub struct Solution {
    pub bins: Vec<Container>,
    pub unplaced: Vec<usize>,
}

impl Solution {
    pub fn bins_used(&self) -> usize {
        self.bins.len()
    }
}

/// Decode a random-key vector into a packing.
///
/// Items are inserted in chromosome order with their forced orientation, using
/// first-fit across the open bins (honoring conflicts via
/// `Container::try_insert_oriented`); a new bin is opened when no open bin accepts
/// the item. An item that cannot fit even a fresh, empty bin in its forced
/// orientation is recorded in `unplaced`.
pub fn decode(instance: &Instance, keys: &[f32]) -> Solution {
    let n = instance.items.len();
    let Decoded { order, rotated } = interpret(keys, n);
    let mut bins: Vec<Container> = Vec::new();
    let mut unplaced: Vec<usize> = Vec::new();

    for idx in order {
        let item = &instance.items[idx];
        let rot = rotated[idx];
        // First-fit into an already-open bin.
        let mut placed = bins.iter_mut().any(|b| {
            b.try_insert_oriented(item, idx, rot, &instance.conflicts)
                .is_some()
        });
        // Otherwise open a new bin.
        if !placed {
            let mut bin = Container::new(instance.width, instance.height);
            if bin
                .try_insert_oriented(item, idx, rot, &instance.conflicts)
                .is_some()
            {
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

/// Fitness over the BRKGA encoding: minimize the number of bins used. Each
/// unplaced item is penalized more than any feasible packing could cost (a
/// feasible packing uses at most `n` bins), so feasibility always dominates.
#[derive(Clone, Debug)]
pub struct BinFitness {
    instance: Arc<Instance>,
}

impl BinFitness {
    pub fn new(instance: Arc<Instance>) -> Self {
        Self { instance }
    }
}

impl Fitness for BinFitness {
    type Genotype = RangeGenotype<f32>;
    fn calculate_for_chromosome(
        &mut self,
        chromosome: &FitnessChromosome<Self>,
        _genotype: &FitnessGenotype<Self>,
    ) -> Option<FitnessValue> {
        let sol = decode(&self.instance, &chromosome.genes);
        let n = self.instance.items.len();
        Some((sol.bins_used() + sol.unplaced.len() * (n + 1)) as FitnessValue)
    }
}

/// Run Evolve on an instance and return the decoded best packing.
pub fn solve(instance: &Instance) -> Solution {
    let n = instance.items.len();
    let genotype = build_genotype(n);
    let evolve = Evolve::builder()
        .with_genotype(genotype)
        .with_target_population_size(100)
        .with_max_stale_generations(200)
        .with_fitness(BinFitness::new(Arc::new(instance.clone())))
        .with_fitness_ordering(FitnessOrdering::Minimize)
        .with_select(SelectTournament::new(0.5, 0.02, 4))
        .with_crossover(CrossoverUniform::new(0.7, 0.8))
        .with_mutate(MutateSingleGene::new(0.2))
        .with_rng_seed_from_u64(0)
        .call()
        .unwrap();
    let (genes, _score) = evolve.best_genes_and_fitness_score().unwrap();
    decode(instance, &genes)
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

    /// Build an instance with the given items and conflict edges.
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

    // Keys: order all upright, natural order 0,1 (n=2).
    fn upright_keys(n: usize) -> Vec<f32> {
        let mut keys = vec![0.0; 2 * n];
        for i in 0..n {
            keys[i] = i as f32 / n as f32; // ascending -> order 0,1,2,...
            keys[n + i] = 0.0; // upright
        }
        keys
    }

    #[test]
    fn conflicting_items_force_separate_bins() {
        let inst = instance(10, 10, vec![item(2, 2), item(2, 2)], &[(0, 1)]);
        let sol = decode(&inst, &upright_keys(2));
        assert_eq!(sol.bins_used(), 2);
        assert!(sol.unplaced.is_empty());
    }

    #[test]
    fn non_conflicting_items_share_a_bin() {
        let inst = instance(10, 10, vec![item(2, 2), item(2, 2)], &[]);
        let sol = decode(&inst, &upright_keys(2));
        assert_eq!(sol.bins_used(), 1);
    }

    #[test]
    fn forced_orientation_is_honored() {
        // A 4x2 item forced rotated lands as a 2x4 footprint.
        let inst = instance(10, 10, vec![item(4, 2)], &[]);
        let mut keys = upright_keys(1);
        keys[1] = 0.9; // orientation key for item 0: rotated
        let sol = decode(&inst, &keys);
        assert_eq!(sol.bins_used(), 1);
        let p = &sol.bins[0].placements()[&0];
        assert!(p.rotated);
        assert_eq!(p.rect.width(), 2);
        assert_eq!(p.rect.height(), 4);
    }

    #[test]
    fn infeasible_forced_orientation_is_penalized() {
        // A 10x3 item fits a 10x3 bin upright, but rotated (3x10) is 10 tall > 3.
        let inst = instance(10, 3, vec![item(10, 3)], &[]);
        let mut keys = upright_keys(1);
        keys[1] = 0.9; // force rotated -> cannot fit
        let sol = decode(&inst, &keys);
        assert_eq!(sol.bins_used(), 0);
        assert_eq!(sol.unplaced, vec![0]);
    }

    #[test]
    fn solve_finds_a_feasible_packing() {
        // Three small non-conflicting items fit one 10x10 bin.
        let inst = instance(10, 10, vec![item(2, 2), item(2, 2), item(2, 2)], &[]);
        let n = inst.items.len();
        let sol = solve(&inst);
        assert!(sol.unplaced.is_empty());
        assert!(
            sol.bins_used() <= n,
            "feasible solution uses at most n bins"
        );
    }
}
