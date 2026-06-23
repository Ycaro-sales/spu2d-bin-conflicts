use std::collections::{HashMap, HashSet};

use crate::{
    conflict_graph::ConflictGraph,
    ems::{Ems, Point},
    item::Item,
};

pub struct Placement {
    pub origin: Point,
    pub rect: Ems,
    pub rotated: bool,
}

pub struct Container {
    width: u32,
    height: u32,
    free: HashSet<Ems>,
    /// Keyed by item index; the key guarantees one placement per item.
    placed: HashMap<usize, Placement>,
}

impl Container {
    pub fn new(width: u32, height: u32) -> Self {
        let whole = Ems::new(
            Point { x: 0, y: 0 },
            Point {
                x: width,
                y: height,
            },
        );
        let mut free = HashSet::new();
        free.insert(whole);
        Self {
            width,
            height,
            free,
            placed: HashMap::new(),
        }
    }

    /// Greedily insert `item` (identified by `index`) into this container, honoring
    /// conflicts. Fails (returns `None`) if `index` conflicts with any already-placed
    /// item, or if it does not fit geometrically. On success returns the placement
    /// origin (delegating geometry to `place`: bottom-left + rotation).
    pub fn try_insert(
        &mut self,
        item: &Item,
        index: usize,
        conflicts: &ConflictGraph<usize>,
    ) -> Option<Point> {
        // Conflict check: any neighbor of `index` already in this container blocks it.
        if conflicts.neighbors(&index).any(|n| self.placed.contains_key(n)) {
            return None;
        }
        self.place(item, index)
    }

    /// Place `item` (identified by `index`) at its bottom-left position using a
    /// caller-fixed orientation, rather than auto-rotating like `place`. Used by the
    /// BRKGA decoder, where orientation is part of the chromosome.
    pub fn place_oriented(&mut self, item: &Item, index: usize, rotated: bool) -> Option<Point> {
        if self.placed.contains_key(&index) {
            return None; // already placed — refuse duplicate
        }
        let (w, h) = if rotated {
            (item.height, item.width)
        } else {
            (item.width, item.height)
        };
        // Bottom-left over the single chosen orientation: lowest origin (y, then x).
        let mut best: Option<Point> = None;
        for ems in &self.free {
            if ems.fits(w, h) {
                let o = ems.min;
                if best.map_or(true, |b| (o.y, o.x) < (b.y, b.x)) {
                    best = Some(o);
                }
            }
        }
        let origin = best?;
        let occupied = Ems::new(
            origin,
            Point {
                x: origin.x + w,
                y: origin.y + h,
            },
        );
        self.split_around(&occupied);
        self.placed.insert(
            index,
            Placement {
                origin,
                rect: occupied,
                rotated,
            },
        );
        Some(origin)
    }

    /// Conflict-aware insert with a caller-fixed orientation (mirrors `try_insert`,
    /// but delegates geometry to `place_oriented`).
    pub fn try_insert_oriented(
        &mut self,
        item: &Item,
        index: usize,
        rotated: bool,
        conflicts: &ConflictGraph<usize>,
    ) -> Option<Point> {
        if conflicts.neighbors(&index).any(|n| self.placed.contains_key(n)) {
            return None;
        }
        self.place_oriented(item, index, rotated)
    }

    /// Read-only view of the placements in this container, keyed by item index.
    pub fn placements(&self) -> &HashMap<usize, Placement> {
        &self.placed
    }

    pub fn place(&mut self, item: &Item, index: usize) -> Option<Point> {
        if self.placed.contains_key(&index) {
            return None; // already placed — refuse duplicate
        }
        // Try both orientations; the non-rotated one is first so it wins ties.
        let orientations = [(item.width, item.height), (item.height, item.width)];
        // Bottom-left: pick the candidate with the lowest origin (y, then x).
        let mut best: Option<(Point, u32, u32)> = None;
        for ems in &self.free {
            for &(w, h) in &orientations {
                if ems.fits(w, h) {
                    let origin = ems.min;
                    let better = match best {
                        None => true,
                        Some((b, _, _)) => (origin.y, origin.x) < (b.y, b.x),
                    };
                    if better {
                        best = Some((origin, w, h));
                    }
                }
            }
        }
        let (origin, w, h) = best?;
        let occupied = Ems::new(
            origin,
            Point {
                x: origin.x + w,
                y: origin.y + h,
            },
        );
        let rotated = w != item.width;
        self.split_around(&occupied);
        self.placed.insert(
            index,
            Placement {
                origin,
                rect: occupied,
                rotated,
            },
        );
        Some(origin)
    }

    fn split_around(&mut self, item: &Ems) {
        let mut next = Vec::new();
        for ems in &self.free {
            if !ems.intersects(item) {
                next.push(ems.clone());
                continue;
            }
            // 4 candidate slabs, each spanning the full opposite dimension of `ems`
            // left
            if item.min.x > ems.min.x {
                next.push(Ems::new(
                    ems.min,
                    Point {
                        x: item.min.x,
                        y: ems.max.y,
                    },
                ));
            }
            // right
            if item.max.x < ems.max.x {
                next.push(Ems::new(
                    Point {
                        x: item.max.x,
                        y: ems.min.y,
                    },
                    ems.max,
                ));
            }
            // bottom
            if item.min.y > ems.min.y {
                next.push(Ems::new(
                    ems.min,
                    Point {
                        x: ems.max.x,
                        y: item.min.y,
                    },
                ));
            }
            // top
            if item.max.y < ems.max.y {
                next.push(Ems::new(
                    Point {
                        x: ems.min.x,
                        y: item.max.y,
                    },
                    ems.max,
                ));
            }
        }
        self.free = Self::prune(next);
    }

    /// Drop degenerate rects and any EMS contained in another (keeps them *maximal*).
    fn prune(spaces: Vec<Ems>) -> HashSet<Ems> {
        let kept: Vec<Ems> = spaces
            .into_iter()
            .filter(|e| e.width() > 0 && e.height() > 0)
            .collect();
        kept.iter()
            .enumerate()
            .filter(|(i, e)| {
                !kept
                    .iter()
                    .enumerate()
                    .any(|(j, other)| *i != j && e.contained_in(other) && (e != &other || i > &j))
            })
            .map(|(_, e)| e.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conflict_graph::ConflictGraph;

    fn item(width: u32, height: u32) -> Item {
        Item {
            height,
            width,
            client: 0,
        }
    }

    #[test]
    fn stacks_bottom_left() {
        let mut c = Container::new(4, 4);
        let a = item(4, 1);
        let b = item(4, 1);
        assert_eq!(c.place(&a, 0), Some(Point { x: 0, y: 0 }));
        // Lowest available row is y == 1, directly above the first item.
        assert_eq!(c.place(&b, 1), Some(Point { x: 0, y: 1 }));
        assert!(!c.placed[&0].rotated);
    }

    #[test]
    fn rotates_to_fit() {
        // 2x4 container; a 4x2 item only fits rotated to 2x4.
        let mut c = Container::new(2, 4);
        let tall = item(4, 2);
        assert_eq!(c.place(&tall, 0), Some(Point { x: 0, y: 0 }));
        assert!(c.placed[&0].rotated);
    }

    #[test]
    fn rejects_duplicate_index() {
        let mut c = Container::new(4, 4);
        let a = item(2, 2);
        let b = item(1, 1);
        let first = c.place(&a, 0);
        assert!(first.is_some());
        // Same index again: refused, original placement untouched.
        assert_eq!(c.place(&b, 0), None);
        assert_eq!(c.placed.len(), 1);
        assert_eq!(c.placed[&0].rect, a_rect());
    }

    fn a_rect() -> Ems {
        Ems::new(Point { x: 0, y: 0 }, Point { x: 2, y: 2 })
    }

    #[test]
    fn try_insert_rejects_conflicting_item() {
        let mut c = Container::new(10, 10);
        let mut g = ConflictGraph::new();
        g.add_conflict(0usize, 1usize); // items 0 and 1 conflict

        let it = item(2, 2);
        assert!(c.try_insert(&it, 0, &g).is_some());
        // Item 1 conflicts with the already-placed item 0: refused.
        assert_eq!(c.try_insert(&it, 1, &g), None);
        assert_eq!(c.placed.len(), 1);
        assert!(!c.placed.contains_key(&1));
        // Item 2 has no conflict edge: accepted.
        assert!(c.try_insert(&it, 2, &g).is_some());
    }

    #[test]
    fn try_insert_without_conflicts_behaves_like_place() {
        let mut c = Container::new(10, 10);
        let g: ConflictGraph<usize> = ConflictGraph::new();
        let it = item(3, 3);
        assert_eq!(c.try_insert(&it, 0, &g), Some(Point { x: 0, y: 0 }));
        assert_eq!(c.placed.len(), 1);
    }

    #[test]
    fn try_insert_still_gated_by_geometry() {
        let mut c = Container::new(2, 2);
        let g: ConflictGraph<usize> = ConflictGraph::new();
        let big = item(5, 5); // doesn't fit, no conflict involved
        assert_eq!(c.try_insert(&big, 0, &g), None);
        assert!(c.placed.is_empty());
    }
}
