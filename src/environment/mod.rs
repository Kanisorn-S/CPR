mod cell;

use std::thread::current;
use cell::Cell;
use crate::util::Coord;
use crate::robot::Team;
use crate::robot::Direction::{Left, Right, Up, Down};
use crate::robot::Robot;

pub struct World {
    grid: Vec<Vec<Cell>>,
    width: u8,
    height: u8,
    red_score: u8,
    blue_score: u8,
    red_deposit_box: Coord,
    blue_deposit_box: Coord,
}

impl World {
    pub fn new(width: u8, height: u8, p_gold: f64, max_gold: u8, n_robots: u8) -> Self {
        let mut grid: Vec<Vec<Cell>> = Vec::new();
        for y in (0u8..height).rev() {
            let mut row: Vec<Cell> = Vec::new();
            for x in 0u8..width {
                row.push(Cell::new((x, y), p_gold, max_gold));
            }
            grid.push(row);
        }
        let (red_deposit_box, blue_deposit_box) = World::spawn_deposit_box(width, height, &mut grid);
        Self::spawn_robots(width, height, &mut grid, n_robots);
        Self {
            grid,
            width,
            height,
            red_deposit_box,
            blue_deposit_box,
            red_score: 0,
            blue_score: 0,
        }
    }

    pub fn print_grid(&self) {
        for row in &self.grid {
            for cell in row {
                print!("{:?} ", cell);
            }
            println!();
        }
    }

    fn spawn_deposit_box(width: u8, height: u8, grid: &mut Vec<Vec<Cell>>) -> (Coord, Coord) {
        let red_deposit_box = Coord::random(0..width, 0..height);
        let mut blue_deposit_box: Coord;
        loop {
            blue_deposit_box = Coord::random(0..width, 0..height);
            if blue_deposit_box != red_deposit_box {
                break;
            }
        }
        let Coord { x: x_red, y: y_red } = red_deposit_box;
        let Coord { x: x_blue, y: y_blue } = blue_deposit_box;
        grid[(height - 1 - y_red) as usize][x_red as usize].set_deposit_box(Team::Red);
        grid[(height - 1 - y_blue) as usize][x_blue as usize].set_deposit_box(Team::Blue);
        (red_deposit_box, blue_deposit_box)
    }

    fn spawn_robots(width: u8, height: u8, grid: &mut Vec<Vec<Cell>>, n_robots: u8) {
        // Blue team robots
        for i in 0..n_robots {
            let id = (b'a' + i) as char;
            let current_pos = Coord::random(0..width, 0..height);
            let new_robot = Self::create_robot(id, Team::Blue, current_pos, width, height);
            grid[(height - 1 - current_pos.y) as usize][current_pos.x as usize].add_bot(new_robot);
        }
        
        // Red team robots
        for i in 0..n_robots {
            let id = (b'A' + i) as char;
            let current_pos = Coord::random(0..width, 0..height);
            let new_robot = Self::create_robot(id, Team::Red, current_pos, width, height);
            grid[(height - 1 - current_pos.y) as usize][current_pos.x as usize].add_bot(new_robot);
        }
    }
    
    fn create_robot(id: char, team: Team, current_pos: Coord, width: u8, height: u8) -> Robot {
        let facing = match rand::random_range(0..4) {
            0 => Left,
            1 => Right,
            2 => Down,
            _ => Up,
        };
        Robot::new(id, team, current_pos, facing)
    }
}
