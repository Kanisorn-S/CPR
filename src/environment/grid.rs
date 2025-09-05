use std::fmt::{Debug, Formatter};
use crate::environment::cell::Cell;
use colored::Colorize;
use crate::robot::Robot;
use crate::util::Coord;

pub struct Grid {
    grid: Vec<Vec<Cell>>,
    width: usize,
    height: usize,
}

// Constructor and getters
impl Grid {
    pub fn new(grid: Vec<Vec<Cell>>, width: usize, height: usize) -> Grid {
        Grid {
            grid,
            width,
            height,
        }
    }

    pub fn get_grid(&self) -> &Vec<Vec<Cell>> {
        &self.grid
    }

    pub fn get_mut_cell(&mut self, coord: Coord) -> Option<&mut Cell> {
        let Coord { x, y } = coord;
        match self.grid.get(self.height - y - 1) {
            Some(row) => {
                match row.get(x) {
                    Some(_) => Some(&mut self.grid[self.height - y - 1][x]),
                    None => None
                }
            },
            None => None
        }
    }
    
    pub fn get_cell(&mut self, coord: Coord) -> Option<&Cell> {
        let Coord { x, y } = coord;
        match self.grid.get(self.height - y - 1) {
            Some(row) => {
                match row.get(x) {
                    Some(_) => Some(&self.grid[self.height - y - 1][x]),
                    None => None
                }
            },
            None => None
        }
    }

    pub fn get_width(&self) -> usize {
        self.width
    }
    pub fn get_height(&self) -> usize {
        self.height
    }
}

// Robot logic
impl Grid {
    pub fn add_robot(&mut self, robot: &Robot, coord: Coord) {
        let cell = self.get_mut_cell(coord);
        match cell {
            Some(cell_ref) => {
                cell_ref.add_bot(robot);
            },
            None => {}
        }
    }

    pub fn remove_robot(&mut self, robot: &Robot, coord: Coord) {
        let cell = self.get_mut_cell(coord);
        match cell {
            Some(cell_ref) => {
                cell_ref.remove_bot(robot);
            },
            None => {}
        }
    }
}

// Print functions
impl Debug for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (index, row) in self.grid.iter().enumerate() {
            write!(f, " {} ", (self.height - index - 1).to_string().bold())?;
            for cell in row {
                write!(f, "{:?} ", cell)?;
            }
            writeln!(f)?;
        }
        write!(f, "   ")?;
        for i in 0..self.width {
            write!(f, "    {}     ", i.to_string().bold())?;
        }
        write!(f, "")
    }
}

