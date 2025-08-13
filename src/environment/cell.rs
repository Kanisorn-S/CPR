use std::fmt::{Debug, Formatter};
use crate::robot::{Robot, Team};
use colored::Colorize;
use crate::environment::World;
use crate::util::Coord;

enum CellContent {
    GoldBars(u8),
    DepositBox(Team, u8),
}

pub struct Cell {
    pub coord: Coord,
    red_robots: u8,
    blue_robots: u8,
    content: Option<CellContent>,
}

impl Debug for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let content = match &self.content {
            Some(CellContent::GoldBars(n)) => format!(" {} ", n).bright_yellow().italic(),
            Some(CellContent::DepositBox(Team::Red, n)) => format!("[{}]", n).to_string().red().bold(),
            Some(CellContent::DepositBox(Team::Blue, n)) => format!("[{}]", n).to_string().blue().bold(),
            None => "   ".to_string().green(),
        };
        let red_robots_string = if self.red_robots > 0 {
            self.red_robots.to_string().bright_red().bold()
        } else {
            self.red_robots.to_string().red().dimmed()
        };
        let blue_robots_string = if self.blue_robots > 0 {
            self.blue_robots.to_string().bright_blue().bold()
        } else {
            self.blue_robots.to_string().blue().dimmed()
        };

        write!(f, "({} {} {})", red_robots_string, content, blue_robots_string)
    }
}
impl Cell {
    pub fn new(coord: (usize, usize), p_gold: f64, max_gold: u8) -> Self {
        let contain_gold = rand::random_bool(p_gold);
        let content = if contain_gold {
            Some(CellContent::GoldBars(rand::random_range(1..=max_gold)))
        } else {
            None
        };
        Cell {
            coord: Coord::new(coord.0, coord.1),
            red_robots: 0,
            blue_robots: 0,
            content,
        }
    }

    pub fn set_deposit_box(&mut self, team: Team) {
        self.content = Some(CellContent::DepositBox(team, 0));
    }

    pub fn add_bot(&mut self, robot: &Robot) {
        let team = robot.get_team();
        match team {
            Team::Red => self.red_robots += 1,
            Team::Blue => self.blue_robots += 1,
        }
    }
    
    pub fn remove_bot(&mut self, robot: &Robot) {
        let team = robot.get_team();
        match team {
            Team::Red => self.red_robots -= 1,
            Team::Blue => self.blue_robots -= 1,
        }
    }
    
    pub fn get_gold_amount(&self) -> Option<u8> {
        match self.content {
            Some(CellContent::GoldBars(n)) if n > 0 => Some(n),
            _ => None,
        }
    }
    
    pub fn remove_gold(&mut self) {
        match self.content {
            Some(CellContent::GoldBars(n)) if n > 1 => self.content = Some(CellContent::GoldBars(n - 1)),
            Some(CellContent::GoldBars(_)) => self.content = None,
            _ => ()
        }
    }

    pub fn add_gold(&mut self) {
        match self.content {
            Some(CellContent::GoldBars(n)) => self.content = Some(CellContent::GoldBars(n + 1)),
            Some(CellContent::DepositBox(team, n)) => {
                self.content = Some(CellContent::DepositBox(team, n + 1));
            },
            None => self.content = Some(CellContent::GoldBars(1))
        }
    }

    pub fn is_deposit_box(&self) -> Option<Team> {
        return if let Some(CellContent::DepositBox(team, _)) = self.content {
            Some(team)
        } else {
            None
        }
    }
    
    pub fn increment_score(&mut self) {
        match self.content {
            Some(CellContent::DepositBox(team, n)) => {
                self.content = Some(CellContent::DepositBox(team, n + 1));
            },
            _ => ()
        }
    }
}
