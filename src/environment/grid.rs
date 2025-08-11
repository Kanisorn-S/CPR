use std::fmt::{Debug, Formatter};
use crate::environment::cell::Cell;

pub struct Grid {
    grid: Vec<Vec<Cell>>
}

impl Grid {
    pub fn new(grid: Vec<Vec<Cell>>) -> Grid {
        Grid { grid }
    }

    pub fn get_grid(&self) -> &Vec<Vec<Cell>> {
        &self.grid
    }
}

impl Debug for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for row in self.grid.iter() {
            for cell in row {
                write!(f, "{:?} ", cell)?;
            }
            writeln!(f)?;
        }
        write!(f, "")
    }
}