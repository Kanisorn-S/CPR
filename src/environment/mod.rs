pub mod cell;
pub mod grid;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use cell::Cell;
use crate::environment::grid::Grid;
use crate::util::Coord;
use crate::robot::{Action, Team};
use crate::robot::Direction::{Left, Right, Up, Down};
use crate::robot::Robot;
use colored::Colorize;
use crate::communication::message::{MessageBoard, MessageBox};
use crate::config::logger::LoggerConfig;
use crate::robot::manager::{RobotManager};

pub struct World {
    manual: bool,
    grid: Grid,
    width: usize,
    height: usize,
    red_score: u8,
    blue_score: u8,
    red_deposit_box: Coord,
    blue_deposit_box: Coord,
    pick_up_check: HashMap<Coord, Vec<(char, Team)>>,
    red_team: RobotManager,
    blue_team: RobotManager,
    
    logger_config: LoggerConfig,
}

// Constructor and Getters
impl World {
    pub fn new(width: usize, height: usize, p_gold: f64, max_gold: u8, n_robots: u8, manual: bool) -> Self {
        let mut grid: Vec<Vec<Cell>> = Vec::new();
        for y in (0..height).rev() {
            let mut row: Vec<Cell> = Vec::new();
            for x in 0..width {
                row.push(Cell::new((x, y), p_gold, max_gold));
            }
            grid.push(row);
        }
        let mut grid = Grid::new(grid, width, height);
        let (red_deposit_box, blue_deposit_box) = Self::spawn_deposit_box(width, height, &mut grid);
        let (blue_team, blue_message_board) = Self::spawn_robots(width, height, &mut grid, n_robots, Team::Blue, blue_deposit_box);
        let (red_team, red_message_board) = Self::spawn_robots(width, height, &mut grid, n_robots, Team::Red, red_deposit_box);
        Self {
            manual,
            grid,
            width,
            height,
            red_deposit_box,
            blue_deposit_box,
            red_score: 0,
            blue_score: 0,
            pick_up_check: HashMap::new(),
            red_team: RobotManager::new(Team::Red, red_team, red_message_board),
            blue_team: RobotManager::new(Team::Blue, blue_team, blue_message_board),
            logger_config: LoggerConfig::new(),
        }
    }

    pub fn get_grid(&self) -> &Grid {
        &self.grid
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn get_red_deposit_box(&self) -> &Coord {
        &self.red_deposit_box
    }

    pub fn get_blue_deposit_box(&self) -> &Coord {
        &self.blue_deposit_box
    }

    pub fn get_red_score(&self) -> u8 {
        self.red_score
    }

    pub fn get_blue_score(&self) -> u8 {
        self.blue_score
    }

    pub fn get_red_team(&self) -> &RobotManager {
        &self.red_team
    }

    pub fn get_blue_team(&self) -> &RobotManager {
        &self.blue_team
    }

}

// Initialization functions
impl World {
    fn spawn_deposit_box(width: usize, height: usize, grid: &mut Grid) -> (Coord, Coord) {
        let red_deposit_box = Coord::random(0..width, 0..height);
        let mut blue_deposit_box: Coord;
        loop {
            blue_deposit_box = Coord::random(0..width, 0..height);
            if blue_deposit_box != red_deposit_box {
                break;
            }
        }
        grid.get_mut_cell(red_deposit_box).unwrap().set_deposit_box(Team::Red);
        grid.get_mut_cell(blue_deposit_box).unwrap().set_deposit_box(Team::Blue);
        (red_deposit_box, blue_deposit_box)
    }

    fn spawn_robots(width: usize, height: usize, grid: &mut Grid, n_robots: u8, team: Team, deposit_box: Coord) -> (HashMap<char, Robot>, Arc<Mutex<MessageBoard>>) {
        let mut robots: HashMap<char, Robot> = HashMap::new();
        let message_board: Arc<Mutex<MessageBoard>> = Arc::new(Mutex::new(MessageBoard::new()));
        let first_id = match team {
            Team::Red => b'A',
            Team::Blue => b'a',
        };
        for i in 0..n_robots {
            let id = (first_id + i) as char;
            message_board.lock().unwrap().insert(id, MessageBox::new());
            let current_pos = Coord::random(0..width, 0..height);
            let facing = match rand::random_range(0..4) {
                0 => Left,
                1 => Right,
                2 => Down,
                _ => Up,
            };
            let new_robot = Robot::new(id, team, current_pos, facing, Arc::clone(&message_board), deposit_box);
            grid.get_mut_cell(current_pos).unwrap().add_bot(&new_robot);
            robots.insert(id, new_robot);
        }
        (robots, message_board)
    }
}

// Decisions and Actions
impl World {

    pub fn next_turn(&mut self) {
        self.make_decision(Team::Blue);
        println!();
        self.make_decision(Team::Red);

        println!();

        self.pick_up_check.clear();
        self.take_actions(Team::Blue);
        println!();
        self.take_actions(Team::Red);

        self.check_pickup_logic();
        self.check_fumble();
        self.check_drop_deposit();

        // println!();
        // self.blue_team.print_message_board_debug();

        self.blue_team.update_message_board();
        self.red_team.update_message_board();

        if (self.logger_config.message_board) {
            println!();
            self.blue_team.print_message_board();
            self.red_team.print_message_board();
        }

    }
    pub fn make_decision(&mut self, team: Team) {
        if (self.logger_config.robot_observation) {
            match team {
                Team::Red => println!("{}{:?} {}", "|".red(), team, "Robots Observations".bold()),
                Team::Blue => println!("{}{:?} {}", "|".blue(), team, "Robots Observations".bold()),
            }
        }
        let robot_manager = match team {
            Team::Red => &mut self.red_team,
            Team::Blue => &mut self.blue_team,
        };
        for robot in robot_manager.get_robots() {
            let observations = robot.observable_cells(self.width, self.height);
            robot.observe(&mut self.grid);
            if (self.logger_config.robot_observation) {
                match team {
                    Team::Red => println!("{}    It can currently observe: {:?}", "|".red(), observations),
                    Team::Blue => println!("{}    It can currently observe: {:?}", "|".blue(), observations)
                }
            }
        }
    }

    pub fn take_actions(&mut self, team: Team) {
        if (self.logger_config.robot_decision) {
            match team {
                Team::Red => println!("{}{:?} {}", "|".red(), team, "Robots Decisions".bold()),
                Team::Blue => println!("{}{:?} {}", "|".blue(), team, "Robots Decisions".bold()),
            }
        }
        let robot_manager = match team {
            Team::Red => &mut self.red_team,
            Team::Blue => &mut self.blue_team,
        };
        for robot in robot_manager.get_robots() {
            let action = robot.make_decision(self.manual);
            if let Action::PickUp = action {
                self.pick_up_check.entry(robot.get_coord()).or_insert(Vec::new()).push((robot.get_id(), team));
            }
            if (self.logger_config.robot_decision) {
                match team {
                    Team::Red => println!("{}{:?} Robot {:?} decided to {:?}", "|".red(), team, robot, action),
                    Team::Blue => println!("{}{:?} Robot {:?} decided to {:?}", "|".blue(), team, robot, action)
                }
            }
            robot.take_action(&action, &mut self.grid);
        }

    }
}

// Pickup Logic
impl World {
    fn check_pickup_logic(&mut self) {
        for (coord, robots) in &self.pick_up_check {
            let gold_bars = self.grid.get_mut_cell(*coord).unwrap().get_gold_amount();
            match gold_bars {
                Some(n) => {
                    if robots.len() < 2 {
                        continue;
                    } else {
                        let mut reds: Vec<char> = Vec::new();
                        let mut blues: Vec<char> = Vec::new();
                        for (id, team) in robots {
                            match *team {
                                Team::Red => reds.push(*id),
                                Team::Blue => blues.push(*id)
                            }
                        }
                        let (red_is_able_to_pick, blue_is_able_to_pick) = Self::teams_that_picks(reds.len(), blues.len(), n);
                        if red_is_able_to_pick {
                            let picked = self.red_team.pickup_gold(reds[0], reds[1]);
                            if picked {
                                self.grid.get_mut_cell(*coord).unwrap().remove_gold();
                                // println!("{}{} and {} has {} picked up a {}", "|".red(), reds[0].to_string().red().bold(), reds[1].to_string().red().bold(), "SUCCESSFULLY".green().bold(), "GOLD BAR".yellow().bold())
                            }
                        }
                        if blue_is_able_to_pick {
                            let picked = self.blue_team.pickup_gold(blues[0], blues[1]);
                            if picked {
                                self.grid.get_mut_cell(*coord).unwrap().remove_gold();
                                // println!("{}{} and {} has {} picked up a {}", "|".blue(), blues[0].to_string().blue().bold(), blues[1].to_string().blue().bold(), "SUCCESSFULLY".green().bold(), "GOLD BAR".yellow().bold())
                            }
                        }
                    }
                },
                None => continue
            }
        }
    }

    fn is_invalid_pickup(red_robots: usize, blue_robots: usize, golds: u8) -> bool {
        (red_robots > 2 && blue_robots > 2) ||
            (red_robots < 2 && blue_robots < 2) ||
            (red_robots == 2 && blue_robots == 2 && golds < 2)
    }

    fn teams_that_picks(red_robots: usize, blue_robots: usize, golds: u8) -> (bool, bool) {
        if Self::is_invalid_pickup(red_robots, blue_robots, golds) {
            (false, false)
        } else {
            (red_robots == 2, blue_robots == 2)
        }
    }
}

// Fumble logic
impl World {
    fn check_fumble(&mut self) {
        let add_gold_coords = self.get_gold_coords();
        for gold_coord in add_gold_coords {
            self.grid.get_mut_cell(gold_coord).unwrap().add_gold();
        }
    }

    fn get_gold_coords(&mut self) -> Vec<Coord> {
        let red_carriers = self.red_team.get_carrying_robot();
        let blue_carriers = self.blue_team.get_carrying_robot();
        let mut add_gold_coords: Vec<Coord> = Vec::new();
        Self::get_drop_coords(red_carriers, &mut add_gold_coords);
        Self::get_drop_coords(blue_carriers, &mut add_gold_coords);
        add_gold_coords
    }

    fn get_drop_coords(carriers: Option<Vec<&mut Robot>>, add_gold_coords: &mut Vec<Coord>) {
        match carriers {
            Some(carriers) => {
                let mut robot_pos: HashMap<char, &mut Robot> = HashMap::new();
                for carrier in carriers {
                    let partner_id = carrier.get_pair_id().unwrap();
                    let partner_coord = robot_pos.remove(&partner_id);
                    match partner_coord {
                        Some(pair_robot) => {
                            let carrier_latest_action = carrier.get_latest_action();
                            let pair_latest_action = pair_robot.get_latest_action();
                            let drop = (carrier_latest_action != pair_latest_action) |
                                       (carrier_latest_action == Action::PickUp && carrier.was_carrying()) |
                                       (pair_latest_action == Action::PickUp && pair_robot.was_carrying());
                            if drop {
                                add_gold_coords.push(carrier.drop_gold());
                                pair_robot.drop_gold();
                            }
                        },
                        None => {
                            robot_pos.insert(carrier.get_id(), carrier);
                        }
                    }
                }
            },
            None => ()
        }
    }

}

// Deposit Logic
impl World {
    fn check_drop_deposit(&mut self) {
        let red_carriers = self.red_team.get_carrying_robot();
        let blue_carriers = self.blue_team.get_carrying_robot();
        match red_carriers {
            Some(carriers) => {
                let mut robot_pos: HashMap<char, &mut Robot> = HashMap::new();
                for carrier in carriers {
                    let partner_id = carrier.get_pair_id().unwrap();
                    let partner_coord = robot_pos.remove(&partner_id);
                    match partner_coord {
                        Some(pair_robot) => {
                            if pair_robot.get_coord() == carrier.get_coord() && carrier.get_coord() == self.red_deposit_box {
                                carrier.score_gold();
                                pair_robot.score_gold();
                                self.red_score += 1;
                                carrier.scored();
                                pair_robot.scored();
                                // println!("{}{}: {}", "|".red(), "RED".red().bold(), self.red_score.to_string().red());
                                // println!("{}{}: {}", "|".blue(), "BLU".blue().bold(), self.blue_score.to_string().blue());
                                self.grid.get_mut_cell(self.red_deposit_box).unwrap().increment_score();
                            }
                        },
                        None => {
                            robot_pos.insert(carrier.get_id(), carrier);
                        }
                    }
                }
            },
            None => ()
        }
        match blue_carriers {
            Some(carriers) => {
                let mut robot_pos: HashMap<char, &mut Robot> = HashMap::new();
                for carrier in carriers {
                    let partner_id = carrier.get_pair_id().unwrap();
                    let partner_coord = robot_pos.remove(&partner_id);
                    match partner_coord {
                        Some(pair_robot) => {
                            if pair_robot.get_coord() == carrier.get_coord() && carrier.get_coord() == self.blue_deposit_box {
                                carrier.score_gold();
                                pair_robot.score_gold();
                                self.blue_score += 1;
                                carrier.scored();
                                pair_robot.scored();
                                // println!("{}{}: {}", "|".red(), "RED".red().bold(), self.red_score.to_string().red());
                                // println!("{}{}: {}", "|".blue(), "BLU".blue().bold(), self.blue_score.to_string().blue());
                                self.grid.get_mut_cell(self.blue_deposit_box).unwrap().increment_score();
                            }
                        },
                        None => {
                            robot_pos.insert(carrier.get_id(), carrier);
                        }
                    }
                }
            },
            None => ()
        }
    }
    
    pub fn increment_score(&mut self, team: Team) {
        match team {
            Team::Blue => self.blue_score += 1,
            Team::Red => self.red_score += 1,
        }
    }
}

// Print functions
impl World {
    pub fn print_grid(&self) {
        println!("{:?}", self.grid);
    }

    pub fn print_pickup_check(&self) {
        println!("Pickup check: {:?}", self.pick_up_check);
    }

    pub fn print_robots(&mut self) {
        for blue_robot in &self.blue_team.get_robots() {
            println!("{}{:?}", "|".blue(), blue_robot);
        }
        for red_robot in &self.red_team.get_robots() {
            println!("{}{:?}", "|".red(), red_robot);
        }
    }
}
