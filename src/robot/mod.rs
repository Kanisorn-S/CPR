pub mod manager;

use std::collections::{LinkedList, HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use std::io;
use crate::util::Coord;
use colored::{ColoredString, Colorize};
use crate::environment::cell::Cell;
use crate::environment::grid::Grid;
use crate::robot::manager::Message;

#[derive(Copy, Clone)]
pub enum Team {
    Red,
    Blue,
}

impl Team {
    pub fn style(&self, text: String) -> ColoredString {
        match self {
            Team::Red => text.red(),
            Team::Blue => text.blue(),
        }
    }
}

impl Debug for Team {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Team::Red => write!(f, "{}", "RED".red()),
            Team::Blue => write!(f, "{}", "BLU".blue()),
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

    // Perception
    observable_cells: LinkedList<Coord>,
    knowledge_base: HashMap<Coord, Cell>,

    // Communication
    message_board: Arc<Mutex<HashMap<char, HashSet<Message>>>>,
    max_id: u32,
    coord_to_send: Option<Coord>,
    increment: u32,
}

// Constructors and getters
impl Robot {
    pub fn new(id: char, team: Team, current_coord: Coord, facing: Direction, message_board: Arc<Mutex<HashMap<char, HashSet<Message>>>>) -> Self {
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
            observable_cells: LinkedList::new(),
            knowledge_base: HashMap::new(),
            message_board,
            max_id: 0,
            coord_to_send: None,
            increment: id as u32,
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


    pub fn is_carrying(&self) -> bool {
        self.is_carrying
    }
    pub fn get_pair_id(&self) -> Option<char> {
        self.pair_id
    }

}

// Decision logic 
impl Robot {
    pub fn make_decision(&mut self, manual: bool) -> Action {
        if (manual) {
            let mut input_string = String::new();
            io::stdin().read_line(&mut input_string).expect("Failed to read line");
            match input_string.trim() {
                "u" => Action::Turn(Direction::Up),
                "d" => Action::Turn(Direction::Down),
                "l" => Action::Turn(Direction::Left),
                "r" => Action::Turn(Direction::Right),
                "p" => Action::PickUp,
                _ => Action:: Move,
            }
        } else {
            match rand::random_range(5..7) {
                1 => Action::Turn(Direction::Left),
                2 => Action::Turn(Direction::Right),
                3 => Action::Turn(Direction::Up),
                4 => Action::Turn(Direction::Down),
                5 => Action::Move,
                _ => Action::PickUp,
            }
        }
    }

}

// Action logic
impl Robot {
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
    
    pub fn pickup(&mut self, pair_id: char) {
        if !self.is_carrying {
            self.is_carrying = true;
            self.pair_id = Some(pair_id);
        }
    }

}

// Gold logic 
impl Robot {
    pub fn drop_gold(&mut self) -> Coord {
        match self.team {
            Team::Red => println!("{}{} has {} a {} at {:?}", "|".red(), self.id.to_string().red().bold(), "DROPPED".on_red().bold().italic(), "GOLD BAR".yellow().bold(), self.coord_history[self.turn - 1]),
            Team::Blue => println!("{}{} has {} a {} at {:?}", "|".blue(), self.id.to_string().blue().bold(), "DROPPED".on_red().bold().italic(), "GOLD BAR".yellow().bold(), self.coord_history[self.turn - 1]),
        }
        self.is_carrying = false;
        self.coord_history[self.turn - 1]
    }

    pub fn score_gold(&mut self) {
        match self.team {
            Team::Red => println!("{}{} has {}", "|".red(), self.id.to_string().red().bold(), "SCORED!".green().bold()),
            Team::Blue => println!("{}{} has {}", "|".blue(), self.id.to_string().blue().bold(), "SCORED!".green().bold()),
        }
        self.is_carrying = false;
    }
}

// Observation logic
impl Robot {

    pub fn observe(&mut self, grid: &mut Grid) {
        for observable_cell in self.observable_cells.iter() {
            let observed_cell = grid.get_cell(*observable_cell).unwrap();
            self.knowledge_base.entry(observed_cell.coord).or_insert(observed_cell);
        }
        match self.team {
            Team::Red => println!("{}{:?} Robot {} Current KB: {:?}", "|".red(), self.team, self.id.to_string().red(), self.knowledge_base),
            Team::Blue => println!("{}{:?} Robot {} Current KB: {:?}", "|".blue(), self.team, self.id.to_string().blue(), self.knowledge_base),

        }
    }
    pub fn observable_cells(&mut self, width: usize, height: usize) -> LinkedList<Coord> {
        let mut observable_cells: LinkedList::<Coord> = LinkedList::new();
        let mut current_coord = self.current_coord;
        match self.facing {
            Direction::Left => {
                if (current_coord.x == 0) {
                    self.observable_cells = observable_cells.clone();
                    return observable_cells;
                }
                current_coord.x -= 1
            },
            Direction::Right => {
                if (current_coord.x == width - 1) {
                    self.observable_cells = observable_cells.clone();
                    return observable_cells;
                }
                current_coord.x += 1
            },
            Direction::Up => {
                if (current_coord.y == height - 1) {
                    self.observable_cells = observable_cells.clone();
                    return observable_cells;
                }
                current_coord.y += 1
            },
            Direction::Down => {
                if (current_coord.y == 0) {
                    self.observable_cells = observable_cells.clone();
                    return observable_cells;
                }
                current_coord.y -= 1
            },
        }
        for i in 0..=1 {
            let x = current_coord.x;
            let y = current_coord.y;
            match self.facing {
                Direction::Left | Direction::Right=> {
                    if (y + i < height) {
                        observable_cells.push_back(Coord::new(x, y + i))
                    }
                },
                Direction::Up | Direction::Down => {
                    if (x + i < width) {
                        observable_cells.push_back(Coord::new(x + i, y))
                    }
                }
            }
        }
        match self.facing {
            Direction::Left => {
                if (current_coord.y != 0) {
                    observable_cells.push_back(Coord::new(current_coord.x, current_coord.y - 1));
                }
                if (current_coord.x == 0) {
                    self.observable_cells = observable_cells.clone();
                    return observable_cells;
                }
                current_coord.x -= 1
            },
            Direction::Right => {
                if (current_coord.y != 0) {
                    observable_cells.push_back(Coord::new(current_coord.x, current_coord.y - 1));
                }
                if (current_coord.x == width - 1) {
                    self.observable_cells = observable_cells.clone();
                    return observable_cells;
                }
                current_coord.x += 1
            },
            Direction::Up => {
                if (current_coord.x != 0) {
                    observable_cells.push_back(Coord::new(current_coord.x - 1, current_coord.y));
                }
                if (current_coord.y == height - 1) {
                    self.observable_cells = observable_cells.clone();
                    return observable_cells;
                }
                current_coord.y += 1
            },
            Direction::Down => {
                if (current_coord.x != 0) {
                    observable_cells.push_back(Coord::new(current_coord.x - 1, current_coord.y));
                }
                if (current_coord.y == 0) {
                    self.observable_cells = observable_cells.clone();
                    return observable_cells;
                }
                current_coord.y -= 1
            },
        }
        for i in (0..=2).rev() {
            let x = current_coord.x;
            let y = current_coord.y;
            match self.facing {
                Direction::Left | Direction::Right=> {
                    if (y >= i) {
                        observable_cells.push_back(Coord::new(x, y - i))}
                    }
                Direction::Up | Direction::Down => {
                    if (x >= i) {
                        observable_cells.push_back(Coord::new(x - i, y))
                    }
                }
            }
        }
        for i in 1..=2 {
            let x = current_coord.x;
            let y = current_coord.y;
            match self.facing {
                Direction::Left | Direction::Right => {
                    if (y + i < height) {
                        observable_cells.push_back(Coord::new(x, y + i))
                    }
                },
            Direction::Up | Direction::Down => {
                if (x + i < width) {
                    observable_cells.push_back(Coord::new(x + i, y))}
                }
            }
        }
        self.observable_cells = observable_cells.clone();
        observable_cells
    }
}

// Conversation Logic
impl Robot {
    fn send(&mut self, message: Message, receiver_ids: Vec<char>) {
        let mut message_board_guard = self.message_board.lock().unwrap();
        for receiver_id in receiver_ids {
            message_board_guard.entry(receiver_id).or_default().insert(message);
        }
    }

    fn receive(&self) -> Option<Message> {
        let mut message_board_guard = self.message_board.lock().unwrap();
        let mut message_to_return = None;
        if let Some(messages) = message_board_guard.get_mut(&self.id) {
            let random_message = messages.iter().next().unwrap().clone();
            messages.remove(&random_message);
            message_to_return = Some(random_message);
        }
        message_to_return
    }
}

// Print functions
impl Debug for Robot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.team {
            Team::Red => {
                write!(f, "{} is at {:?} facing {:?}", self.id.to_string().red(), self.current_coord, self.facing)?;
                if self.is_carrying {
                    write!(f, " is {} with {}", "CARRYING".yellow().bold(), self.pair_id.unwrap().to_string().red().dimmed())
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

