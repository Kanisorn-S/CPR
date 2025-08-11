use std::fmt::{Debug, Formatter};
use crate::environment::cell::Cell;
use colored::Colorize;
use crate::robot::{Robot, Team};
use crate::util::Coord;

pub struct Grid {
    grid: Vec<Vec<Cell>>,
    width: u8,
    height: u8,
}

impl Grid {
    pub fn new(grid: Vec<Vec<Cell>>, width: u8, height: u8) -> Grid {
        Grid {
            grid,
            width,
            height,
        }

    }

    pub fn get_grid(&self) -> &Vec<Vec<Cell>> {
        &self.grid
    }

    fn get_cell(&mut self, coord: Coord) -> Option<&mut Cell> {
        let Coord { x, y } = coord;
        match self.grid.get(self.height as usize - y as usize - 1) {
            Some(row) => {
                match row.get(x as usize) {
                    Some(_) => Some(&mut self.grid[(self.height - y - 1) as usize][x as usize]),
                    None => None
                }
            },
            None => None
        }
    }

    pub fn add_robot(&mut self, robot: &Robot, coord: Coord) {
        let cell = self.get_cell(coord);
        match cell {
            Some(cell_ref) => {
                cell_ref.add_bot(robot);
            },
            None => {}
        }
    }

    pub fn remove_robot(&mut self, robot: &Robot, coord: Coord) {
        let cell = self.get_cell(coord);
        match cell {
            Some(cell_ref) => {
                cell_ref.remove_bot(robot);
            },
            None => {}
        }
    }

    pub fn get_width(&self) -> u8 {
        self.width
    }
    pub fn get_height(&self) -> u8 {
        self.height
    }
}

impl Debug for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (index, row) in self.grid.iter().enumerate() {
            write!(f, " {} ", (self.height - index as u8 - 1).to_string().bold())?;
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

