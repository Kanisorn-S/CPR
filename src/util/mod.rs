use std::fmt::{Debug, Formatter};
use std::ops::Range;
use colored::Colorize;

#[derive(PartialEq, Clone, Copy, Eq, Hash)]
pub struct Coord {
    pub x: u8,
    pub y: u8,
}

impl Debug for Coord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = format!("({}, {})", self.x, self.y).bold().to_string();
        write!(f, "{}", s)
    }
}

impl Coord {
    pub fn new(x: u8, y: u8) -> Coord {
        Coord { x, y }
    }

    pub fn random(range_x: Range<u8>, range_y: Range<u8>) -> Coord {
        let x = rand::random_range(range_x);
        let y = rand::random_range(range_y);
        Coord { x, y }
    }
}