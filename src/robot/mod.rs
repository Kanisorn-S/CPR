pub mod manager;

use std::fmt::{Debug, Formatter};
use crate::util::Coord;
use colored::Colorize;
use crate::environment::grid::Grid;

#[derive(Copy, Clone)]
pub enum Team {
    Red,
    Blue,
}

impl Debug for Team {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Team::Red => write!(f, "{}", "RED".red()),
            Team::Blue => write!(f, "{}", "BLUE".blue()),
        }
    }
}
#[derive(Copy, Clone)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Debug for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Left => write!(f, "{}", "LEFT".purple()),
            Direction::Right => write!(f, "{}", "RIGHT".purple()),
            Direction::Up => write!(f, "{}", "UP".purple()),
            Direction::Down => write!(f, "{}", "DOWN".purple()),
        }
    }
}


pub enum Action {
    Move,
    Turn(Direction),
    PickUp,
}

impl Debug for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Move => write!(f, "{}", "MOVE".green().bold()),
            Action::Turn(direction) => write!(f, "{} to {:?}", "TURN".red().bold(), direction),
            Action::PickUp => write!(f, "{}", "PICK UP".yellow().bold()),
        }
    }
}

pub struct Robot {
    id: char,
    team: Team,
    current_coord: Coord,
    facing: Direction,
    is_carrying: bool,
    pair_id: Option<char>,
    coord_history: Vec<Coord>,
    action_history: Vec<Action>,
    turn: usize,
}

impl Debug for Robot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.team {
            Team::Red => {
                write!(f, "{} is at {:?} facing {:?}", self.id.to_string().red(), self.current_coord, self.facing)?;
                if self.is_carrying {
                    write!(f, "{} with {}", "CARRYING".yellow().bold(), self.pair_id.unwrap().to_string().red().dimmed())
                } else {
                    write!(f, "")
                }
            },
            Team::Blue => {
                write!(f, "{} is at {:?} facing {:?}", self.id.to_string().blue(), self.current_coord, self.facing)?;
                if self.is_carrying {
                    write!(f, " is {} with {}", "CARRYING GOLD".yellow().bold(), self.pair_id.unwrap().to_string().blue().dimmed())
                } else {
                    write!(f, "")
                }
            },
        }
    }
}

impl Robot {

    pub fn new(id: char, team: Team, current_coord: Coord, facing: Direction) -> Self {
        let mut coord_history: Vec<Coord> = Vec::new();
        coord_history.push(current_coord);
        Robot {
            id,
            team,
            current_coord,
            facing,
            is_carrying: false,
            pair_id: None,
            coord_history,
            action_history: Vec::new(),
            turn: 0,
        }
    }

    pub fn make_decision(&self) -> Action {
        match rand::random_range(5..7) {
            1 => Action::Turn(Direction::Left),
            2 => Action::Turn(Direction::Right),
            3 => Action::Turn(Direction::Up),
            4 => Action::Turn(Direction::Down),
            5 => Action::Move,
            _ => Action::PickUp,
        }
    }

    pub fn get_team(&self) -> Team {
        self.team
    }

    pub fn get_id(&self) -> char {
        self.id
    }

    pub fn get_coord(&self) -> Coord {
        self.current_coord
    }

    pub fn take_action(&mut self, action: &Action, grid: &mut Grid) {
        match action {
            Action::Turn(direction) => {
                self.turn(*direction);
                self.action_history.push(Action::Turn(*direction));
                self.coord_history.push(self.current_coord);
            },
            Action::Move => {
                self.step(grid);
                self.action_history.push(Action::Move);
                self.coord_history.push(self.current_coord);
            },
            Action::PickUp => {
                self.action_history.push(Action::PickUp);
                self.coord_history.push(self.current_coord);
            }
        }
        self.turn += 1;
    }

    fn turn(&mut self, direction: Direction) {
        self.facing = direction;
    }

    fn step(&mut self, grid: &mut Grid) {
        match self.facing {
            Direction::Left => {
                let current_x = self.current_coord.x;
                if current_x > 0 {
                    grid.remove_robot(self, self.current_coord);
                    self.current_coord.x -= 1;
                    grid.add_robot(self, self.current_coord);
                }
            },
            Direction::Right => {
                let current_x = self.current_coord.x;
                if current_x < grid.get_width() - 1 {
                    grid.remove_robot(self, self.current_coord);
                    self.current_coord.x += 1;
                    grid.add_robot(self, self.current_coord);
                }
            },
            Direction::Up => {
                let current_y = self.current_coord.y;
                if current_y < grid.get_height() - 1 {
                    grid.remove_robot(self, self.current_coord);
                    self.current_coord.y += 1;
                    grid.add_robot(self, self.current_coord);
                }
            },
            Direction::Down => {
                let current_y = self.current_coord.y;
                if current_y > 0 {
                    grid.remove_robot(self, self.current_coord);
                    self.current_coord.y -= 1;
                    grid.add_robot(self, self.current_coord);
                }
            },
        }
    }

    pub fn is_carrying(&self) -> bool {
        self.is_carrying
    }
    pub fn pickup(&mut self, pair_id: char) {
        if !self.is_carrying {
            self.is_carrying = true;
            self.pair_id = Some(pair_id);
        }
    }

    pub fn get_pair_id(&self) -> Option<char> {
        self.pair_id
    }

    pub fn drop_gold(&mut self) -> Coord {
        match self.team {
            Team::Red => println!("{} has {} a {} at {:?}", self.id.to_string().red().bold(), "DROPPED".on_red().bold().italic(), "GOLD BAR".yellow().bold(), self.coord_history[self.turn - 1]),
            Team::Blue => println!("{} has {} a {} at {:?}", self.id.to_string().blue().bold(), "DROPPED".on_red().bold().italic(), "GOLD BAR".yellow().bold(), self.coord_history[self.turn - 1]),
        }
        self.is_carrying = false;
        self.coord_history[self.turn - 1]
    }
    
    pub fn score_gold(&mut self) {
        match self.team {
            Team::Red => println!("{} has {}", self.id.to_string().red().bold(), "SCORED!".green().bold()),
            Team::Blue => println!("{} has {}", self.id.to_string().blue().bold(), "SCORED!".green().bold()),
        }
        self.is_carrying = false;
    }
}


