use std::fmt::{Debug, Formatter};
use std::ops::Range;
use colored::Colorize;

#[derive(PartialEq, Clone, Copy, Eq, Hash)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
}


// Constructor and getters
impl Coord {
    pub fn new(x: usize, y: usize) -> Coord {
        Coord { x, y }
    }

    pub fn random(range_x: Range<usize>, range_y: Range<usize>) -> Coord {
        let x = rand::random_range(range_x);
        let y = rand::random_range(range_y);
        Coord { x, y }
    }
}

// Print functions
impl Debug for Coord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = format!("({}, {})", self.x, self.y).bold().to_string();
        write!(f, "{}", s)
    }
}
