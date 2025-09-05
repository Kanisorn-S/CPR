use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use crate::robot::{Robot, Team};
use crate::util::Coord;

pub struct RobotManager {
    team: Team,
    robots: HashMap<char, Robot>,
    message_board: Arc<Mutex<HashMap<char, HashSet<Message>>>>,
}

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum MessageType {
    PrepareRequest,
    PrepareResponse,
    AcceptRequest
}

#[derive(PartialEq, Hash, Eq, Clone, Copy, Debug)]
pub struct Message {
    msg_type: MessageType,
    id: u32,
    coord: Coord,
}

// Constructor and getters
impl RobotManager {
    pub fn new(team: Team, robots: HashMap<char, Robot>, message_board: Arc<Mutex<HashMap<char, HashSet<Message>>>>) -> RobotManager {
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

// Robot Actions logic
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