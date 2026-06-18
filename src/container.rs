use std::collections::HashSet;

use crate::{
    ems::{Ems, Point},
    item::Item,
};

pub struct Container {
    width: u32,
    height: u32,
    free: HashSet<Ems>,
    placed: HashSet<(Point, Ems)>,
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
            placed: HashSet::new(),
        }
    }

    pub fn place(&mut self, item: &Item) -> Option<Point> {
        let (w, h) = (item.width, item.height);
        let origin = self.free.iter().find(|e| e.fits(w, h))?.min;
        let occupied = Ems::new(
            origin,
            Point {
                x: origin.x + w,
                y: origin.y + h,
            },
        );
        self.split_around(&occupied);
        self.placed.insert((origin, occupied));
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
