#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Ems {
    pub min: Point,
    pub max: Point,
}

impl Ems {
    pub fn new(min: Point, max: Point) -> Self {
        Self { min, max }
    }
    pub fn width(&self) -> u32 {
        self.max.x - self.min.x
    }
    pub fn height(&self) -> u32 {
        self.max.y - self.min.y
    }
    pub fn fits(&self, w: u32, h: u32) -> bool {
        self.width() >= w && self.height() >= h
    }
    pub fn contained_in(&self, other: &Ems) -> bool {
        other.min.x <= self.min.x
            && other.min.y <= self.min.y
            && other.max.x <= self.max.x
            && other.max.y <= self.max.y
    }
    pub fn intersects(&self, r: &Ems) -> bool {
        self.min.x < r.max.x && r.min.x < self.max.x && self.min.y < r.max.y && r.min.y < self.max.y
    }
}
