use genetic_algorithm::strategy::evolve::prelude::*;

/// Orientation keys at or above this threshold mean the item is rotated 90°.
pub const ROTATION_THRESHOLD: f32 = 0.5;

/// Build the BRKGA random-key genotype for an instance with `n` items: `2n`
/// continuous keys in `[0, 1]`. Genes `0..n` are insertion-order keys, genes
/// `n..2n` are orientation keys.
pub fn build_genotype(n: usize) -> RangeGenotype<f32> {
    RangeGenotype::builder()
        .with_genes_size(2 * n)
        .with_allele_range(0.0..=1.0)
        .build()
        .unwrap()
}

/// A decoded chromosome: the item insertion order and the per-item orientation.
pub struct Decoded {
    /// Item indices in the order they should be inserted (ascending by key).
    pub order: Vec<usize>,
    /// `rotated[i]` is true when item `i` should be placed rotated 90°.
    pub rotated: Vec<bool>,
}

/// Interpret a `2n` random-key vector for an instance with `n` items.
///
/// The first `n` keys define the insertion order (items sorted by ascending key);
/// the last `n` keys define orientation (`>= ROTATION_THRESHOLD` => rotated).
pub fn interpret(keys: &[f32], n: usize) -> Decoded {
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        keys[a]
            .partial_cmp(&keys[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let rotated = (0..n).map(|i| keys[n + i] >= ROTATION_THRESHOLD).collect();
    Decoded { order, rotated }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_by_ascending_key_and_reads_orientation() {
        // n = 3. Order keys: item0=0.9, item1=0.1, item2=0.5 -> order [1, 2, 0].
        // Orientation keys: item0=0.2 (upright), item1=0.5 (rotated, boundary),
        // item2=0.8 (rotated).
        let keys = [0.9, 0.1, 0.5, 0.2, 0.5, 0.8];
        let d = interpret(&keys, 3);
        assert_eq!(d.order, vec![1, 2, 0]);
        assert_eq!(d.rotated, vec![false, true, true]);
    }
}
