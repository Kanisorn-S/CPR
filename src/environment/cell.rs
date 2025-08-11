use std::fmt::{Debug, Formatter};
use crate::robot::{Robot, Team};
use colored::Colorize;

enum CellContent {
    GoldBars(u8),
    DepositBox(Team, u8),
}

pub struct Cell {
    coord: (u8, u8),
    robots: Vec<Robot>,
    red_robots: u8,
    blue_robots: u8,
    content: Option<CellContent>,
}

impl Debug for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let content = match &self.content {
            Some(CellContent::GoldBars(n)) => format!(" {} ", n).yellow(),
            Some(CellContent::DepositBox(Team::Red, n)) => format!("[{}]", n).to_string().red(),
            Some(CellContent::DepositBox(Team::Blue, n)) => format!("[{}]", n).to_string().blue(),
            None => "   ".to_string().green(),
        };
        write!(f, "({}{}{})", self.red_robots.to_string().red(), content, self.blue_robots.to_string().blue())
    }
}
impl Cell {
    pub fn new(coord: (u8, u8), p_gold: f64, max_gold: u8) -> Self {
        let contain_gold = rand::random_bool(p_gold);
        let content = if contain_gold {
            Some(CellContent::GoldBars(rand::random_range(1..=max_gold)))
        } else {
            None
        };
        Cell {
            coord,
            robots: Vec::new(),
            red_robots: 0,
            blue_robots: 0,
            content,
        }
    }

    pub fn set_deposit_box(&mut self, team: Team) {
        self.content = Some(CellContent::DepositBox(team, 0));
    }

    pub fn add_bot(&mut self, robot: Robot) {
        let team = robot.get_team();
        match team {
            Team::Red => self.red_robots += 1,
            Team::Blue => self.blue_robots += 1,
        }
        self.robots.push(robot);
    }
}
