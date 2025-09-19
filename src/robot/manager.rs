use std::collections::{HashMap};
use std::sync::{Arc, Mutex};
use crate::communication::message::MessageBoard;
use crate::robot::{Robot, Team};


pub struct RobotManager {
    team: Team,
    robots: HashMap<char, Robot>,
    message_board: Arc<Mutex<MessageBoard>>,
}

// Constructor and getters
impl RobotManager {
    pub fn new(team: Team, robots: HashMap<char, Robot>, message_board: Arc<Mutex<MessageBoard>>) -> RobotManager {
        RobotManager {
            team,
            robots,
            message_board,
        }
    }

    pub fn get_robots(&mut self) -> Vec<&mut Robot> {
        self.robots.values_mut().collect()
    }

    pub fn get_robot_by_id(&mut self, id: char) -> Option<&mut Robot> {
        self.robots.get_mut(&id)
    }
    
    pub fn get_carrying_robot(&mut self) -> Option<Vec<&mut Robot>> {
        let mut carrying_robot: Vec<&mut Robot> = Vec::new();
        for robot in self.robots.values_mut() {
            if robot.is_carrying {
                carrying_robot.push(robot);
            }
        }
        if carrying_robot.is_empty() {
            None
        } else {
            Some(carrying_robot)
        }
    }
}

// Robot Actions Logic
impl RobotManager {
    pub fn pickup_gold(&mut self, id_1: char, id_2: char) -> bool {
        let robot_1 = self.get_robot_by_id(id_1).unwrap();
        if robot_1.is_carrying {
            return false;
        }
        let robot_2 = self.get_robot_by_id(id_2).unwrap();
        if robot_2.is_carrying {
            return false;
        }
        robot_2.pickup(id_1);
        let robot_1 = self.get_robot_by_id(id_1).unwrap();
        robot_1.pickup(id_2);
        true
    }

}

// Robot Communication Logic
impl RobotManager {
    pub fn update_message_board(&mut self) {
        let mut message_board_guard = self.message_board.lock().unwrap();
        message_board_guard.update();
    }
}

// Print Functions
impl RobotManager {
    pub fn print_message_board(&self) {
        match self.team {
            Team::Blue => {
                println!("{} Message Board", self.team.style("BLU".to_string()));
                println!("{}", self.message_board.lock().unwrap());
            },
            Team::Red => {
                println!("{} Message Board", self.team.style("RED".to_string()));
                println!("{}", self.message_board.lock().unwrap());
            }
        }
    }
    pub fn print_message_board_debug(&self) {
        match self.team {
            Team::Blue => {
                println!("{} Message Board", self.team.style("BLU".to_string()));
                println!("{:?}", self.message_board.lock().unwrap());
            },
            Team::Red => {
                println!("{} Message Board", self.team.style("RED".to_string()));
                println!("{:?}", self.message_board.lock().unwrap());
            }
        }
    }
}