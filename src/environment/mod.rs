pub mod cell;
pub mod grid;

use std::collections::HashMap;
use cell::Cell;
use crate::environment::grid::Grid;
use crate::util::Coord;
use crate::robot::{Action, Team};
use crate::robot::Direction::{Left, Right, Up, Down};
use crate::robot::Robot;
use colored::Colorize;
use crate::robot::manager::RobotManager;

pub struct World {
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
}

impl World {
    pub fn new(width: usize, height: usize, p_gold: f64, max_gold: u8, n_robots: u8) -> Self {
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
        let (blue_team, red_team) = Self::spawn_robots(width, height, &mut grid, n_robots);
        Self {
            grid,
            width,
            height,
            red_deposit_box,
            blue_deposit_box,
            red_score: 0,
            blue_score: 0,
            pick_up_check: HashMap::new(),
            red_team: RobotManager::new(Team::Red, red_team),
            blue_team: RobotManager::new(Team::Blue, blue_team),
        }
    }

    pub fn print_grid(&self) {
        println!("{:?}", self.grid);
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    fn spawn_deposit_box(width: usize, height: usize, grid: &mut Grid) -> (Coord, Coord) {
        let red_deposit_box = Coord::random(0..width, 0..height);
        let mut blue_deposit_box: Coord;
        loop {
            blue_deposit_box = Coord::random(0..width, 0..height);
            if blue_deposit_box != red_deposit_box {
                break;
            }
        }
        grid.get_cell(red_deposit_box).unwrap().set_deposit_box(Team::Red);
        // grid.get_cell(blue_deposit_box).unwrap().set_deposit_box(Team::Blue);
        grid.get_cell(Coord::new(0, 1)).unwrap().set_deposit_box(Team::Blue);
        (red_deposit_box, blue_deposit_box)
    }

    fn spawn_robots(width: usize, height: usize, grid: &mut Grid, n_robots: u8) -> (HashMap<char, Robot>, HashMap<char, Robot>) {
        let mut blue_robots: HashMap<char, Robot> = HashMap::new();
        let red_robots: HashMap<char, Robot> = HashMap::new();
        // Blue team robots
        for i in 0..n_robots {
            let id = (b'a' + i) as char;
            let current_pos = Coord::random(0..1, 0..1);
            let new_robot = Self::create_robot(id, Team::Blue, current_pos);
            grid.get_cell(current_pos).unwrap().add_bot(&new_robot);
            blue_robots.insert(id, new_robot);
        }

        // Red team robots
        // for i in 0..n_robots {
        //     let id = (b'A' + i) as char;
        //     let current_pos = Coord::random(0..width, 0..height);
        //     let new_robot = Self::create_robot(id, Team::Red, current_pos);
        //     grid.get_cell(current_pos).unwrap().add_bot(&new_robot);
        //     red_robots.insert(id, new_robot);
        // }
        (blue_robots , red_robots)
    }

    fn create_robot(id: char, team: Team, current_pos: Coord) -> Robot {
        let facing = match rand::random_range(3..4) {
            0 => Left,
            1 => Right,
            2 => Down,
            _ => Up,
        };
        Robot::new(id, team, current_pos, facing)
    }

    pub fn make_decisions_and_take_actions(&mut self) {
        println!("{}", "Robots Decisions".bold());
        self.pick_up_check.clear();
        for blue_robot in self.blue_team.get_robots() {
            let action = blue_robot.make_decision();
            if let Action::PickUp = action {
                self.pick_up_check.entry(blue_robot.get_coord()).or_insert(Vec::new()).push((blue_robot.get_id(), Team::Blue));
            }
            println!("{} Robot {:?} decided to {:?}", "BLU".blue(), blue_robot, action);
            blue_robot.take_action(&action, &mut self.grid);
        }

        for red_robot in self.red_team.get_robots() {
            let action = red_robot.make_decision();
            if let Action::PickUp = action {
                self.pick_up_check.entry(red_robot.get_coord()).or_insert(Vec::new()).push((red_robot.get_id(), Team::Red));
            }
            println!("{} Robot {:?} decided to {:?}", "RED".red(), red_robot, action);
            red_robot.take_action(&action, &mut self.grid);
        }

        self.check_pickup_logic();
        self.check_fumble();
        self.check_drop_deposit();
    }

    fn check_pickup_logic(&mut self) {
        for (coord, robots) in &self.pick_up_check {
            let gold_bars = self.grid.get_cell(*coord).unwrap().get_gold_amount();
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
                                self.grid.get_cell(*coord).unwrap().remove_gold();
                                println!("{} and {} has {} picked up a {}", reds[0].to_string().red().bold(), reds[1].to_string().red().bold(), "SUCCESFULLY".green().bold(), "GOLD BAR".yellow().bold())
                            }
                        }
                        if blue_is_able_to_pick {
                            let picked = self.blue_team.pickup_gold(blues[0], blues[1]);
                            if picked {
                                self.grid.get_cell(*coord).unwrap().remove_gold();
                                println!("{} and {} has {} picked up a {}", blues[0].to_string().blue().bold(), blues[1].to_string().blue().bold(), "SUCCESFULLY".green().bold(), "GOLD BAR".yellow().bold())
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

    fn check_fumble(&mut self) {
        let add_gold_coords = self.get_gold_coords();
        for gold_coord in add_gold_coords {
            self.grid.get_cell(gold_coord).unwrap().add_gold();
        }
    }

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
                                println!("{}: {}", "RED".red().bold(), self.red_score.to_string().red());
                                println!("{}: {}", "BLU".blue().bold(), self.blue_score.to_string().blue());
                                self.grid.get_cell(self.red_deposit_box).unwrap().increment_score();
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
                                println!("{}: {}", "RED".red().bold(), self.red_score.to_string().red());
                                println!("{}: {}", "BLU".blue().bold(), self.blue_score.to_string().blue());
                                self.grid.get_cell(self.blue_deposit_box).unwrap().increment_score();
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

    fn get_gold_coords(&mut self) -> Vec<Coord> {
        let red_carriers = self.red_team.get_carrying_robot();
        let blue_carriers = self.blue_team.get_carrying_robot();
        let mut add_gold_coords: Vec<Coord> = Vec::new();
        match red_carriers {
            Some(carriers) => {
                let mut robot_pos: HashMap<char, &mut Robot> = HashMap::new();
                for carrier in carriers {
                    let partner_id = carrier.get_pair_id().unwrap();
                    let partner_coord = robot_pos.remove(&partner_id);
                    match partner_coord {
                        Some(pair_robot) => {
                            if pair_robot.get_coord() != carrier.get_coord() {
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
        match blue_carriers {
            Some(carriers) => {
                let mut robot_pos: HashMap<char, &mut Robot> = HashMap::new();
                for carrier in carriers {
                    let partner_id = carrier.get_pair_id().unwrap();
                    let partner_coord = robot_pos.remove(&partner_id);
                    match partner_coord {
                        Some(pair_robot) => {
                            if pair_robot.get_coord() != carrier.get_coord() {
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
        add_gold_coords
    }

    pub fn print_pickup_check(&self) {
        println!("Pickup check: {:?}", self.pick_up_check);
    }

    pub fn increment_score(&mut self, team: Team) {
        match team {
            Team::Blue => self.blue_score += 1,
            Team::Red => self.red_score += 1,
        }
    }

    pub fn print_robots(&mut self) {
        for red_robot in &self.red_team.get_robots() {
            println!("{:?}", red_robot);
        }
        for blue_robot in &self.blue_team.get_robots() {
            println!("{:?}", blue_robot);
        }
    }
}
