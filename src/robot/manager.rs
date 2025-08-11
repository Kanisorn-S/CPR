use std::collections::HashMap;
use crate::robot::{Robot, Team};

pub struct RobotManager {
    team: Team,
    robots: HashMap<char, Robot>,
}

impl RobotManager {
    pub fn new(team: Team) -> RobotManager {
        RobotManager {
            team,
            robots: HashMap::new(),
        }
    }

    pub fn get_robots(&mut self) -> &HashMap<char, Robot> {
        &self.robots
    }
}