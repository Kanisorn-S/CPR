pub mod manager;

use std::collections::{LinkedList, HashMap};
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use std::io;
use crate::util::Coord;
use colored::{ColoredString, Colorize};
use rand::Rng;
use crate::communication::message::{Message, MessageBoard, MessageContent, MessageType};
use crate::config::logger::LoggerConfig;
use crate::environment::cell::Cell;
use crate::environment::grid::Grid;
use crate::config::Config;

use rand::seq::IndexedRandom;
use crate::robot::Action::Turn;

#[derive(PartialEq, Debug)]
pub enum RobotState {
    ClusterFinding,
    Paxos,
    WaitingForTaskCompletion,
    MovingToTarget,
    AtTarget,
    MovingToDropBox,
}

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
#[derive(Eq, Hash, Copy, Clone, PartialEq)]
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


#[derive(Clone, Copy, PartialEq)]
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
    // General
    id: char,
    team: Team,
    current_coord: Coord,
    facing: Direction,
    is_carrying: bool,
    was_carrying: bool,
    pair_id: Option<char>,
    coord_history: Vec<Coord>,
    action_history: Vec<Action>,
    turn: usize,
    deposit_box_coord: Coord,

    // Perception
    observable_cells: LinkedList<Coord>,
    knowledge_base: HashMap<Coord, Cell>,

    // Communication
    message_board: Arc<Mutex<MessageBoard>>,
    message_to_send: Option<Message>,

    // Local Cluster Identification
    receiver_ids: Vec<char>,
    target_gold: Option<Coord>,
    old_target_gold: Option<Coord>,
    target_gold_amount: u8,
    max_gold_seen: u8,
    send_target: bool,
    local_cluster: Vec<char>,
    clusters: HashMap<(Coord, u8), Vec<char>>,
    not_received_simple: u8,

    // Backup Cluster
    max_gold_receive: u8,
    max_gold_receive_coord: Option<Coord>,
    backup_cluster: Vec<char>,

    // PAXOS
    consensus_coord: Option<Coord>,
    promised_message: Option<Message>,
    max_id_seen: u32,
    max_piggyback_id_seen: u32,
    promise_count: u8,
    piggybacked: bool,
    reached_majority: bool,
    accept_count: u8,
    majority: u8,
    increment: u32,
    send_pair_request: bool,
    consensus_pair: Option<(char, char)>,
    pre_pickup_pair_id: Option<char>,
    accepted: bool,

    // Direction Consensus
    sent_direction_request: bool,
    received_direction: bool,
    turn_direction: Option<Direction>,
    turned: bool,

    // Move Planning
    planned_actions: Vec<Action>,

    // Next Round
    received_begin: bool,

    // Resolve across cluster
    combined_pair_id: Option<u32>,
    send_getout: bool,
    override_target_gold: bool,

    // React to gold getting nabbed
    is_second_check: bool,

    carrying_with_wrong_pair: bool,

    // State Tracking
    current_state: RobotState,

    // Configurations
    logger_config: LoggerConfig,
}

// Constructors and getters
impl Robot {
    pub fn new(id: char, team: Team, current_coord:Coord, facing: Direction, message_board: Arc<Mutex<MessageBoard>>, deposit_box_coord: Coord) -> Self {
        let mut coord_history: Vec<Coord> = Vec::new();
        let Config { n_robots, .. } = Config::new();
        coord_history.push(current_coord);
        Robot {
            // General
            id,
            team,
            current_coord,
            facing,
            is_carrying: false,
            was_carrying: false,
            pair_id: None,
            coord_history,
            action_history: Vec::new(),
            turn: 0,
            deposit_box_coord,

            // Perception
            observable_cells: LinkedList::new(),
            knowledge_base: HashMap::new(),

            // Communication
            message_board,
            message_to_send: Some(Message::new(
                id,
                MessageType::PrepareRequest,
                id as u32,
                MessageContent::Coord(Some(current_coord), Some(0)),
            )),

            // Local Cluster Identification
            receiver_ids: make_vec(n_robots, id, team),
            target_gold: None,
            old_target_gold: None,
            target_gold_amount: 0,
            max_gold_seen: 0,
            send_target: false,
            local_cluster: Vec::new(),
            clusters: HashMap::new(),
            not_received_simple: n_robots - 1,

            // Backup Cluster
            max_gold_receive: 0,
            max_gold_receive_coord: None,
            backup_cluster: Vec::new(),

            // PAXOS
            consensus_coord: None,
            promised_message: None,
            max_id_seen: 0,
            max_piggyback_id_seen: 0,
            promise_count: 0,
            piggybacked: false,
            reached_majority: false,
            accept_count: 0,
            majority: n_robots / 2,
            increment: id as u32,
            send_pair_request: false,
            consensus_pair: None,
            pre_pickup_pair_id: None,
            accepted: false,

            // Direction Consensus
            sent_direction_request: false,
            turn_direction: None,
            received_direction: false,
            turned: false,

            // Move Planning
            planned_actions: Vec::new(),

            // Next Round
            received_begin: true,

            // Resolve across cluster
            combined_pair_id: None,
            send_getout: false,
            override_target_gold: false,

            // React to gold getting nabbed
            is_second_check: false,

            carrying_with_wrong_pair: false,

            // State Tracking
            current_state: RobotState::ClusterFinding,

            // Configuration
            logger_config: LoggerConfig::new(),
        }
    }

    pub fn reset(&mut self) {

        // General
        self.is_carrying = false;
        self.was_carrying = false;
        self.pair_id = None;

        // Local Cluster Identification
        self.target_gold = None;
        self.old_target_gold = None;
        self.target_gold_amount = 0;
        self.max_gold_seen = 0;
        self.send_target = false;
        self.clusters = HashMap::new();
        self.not_received_simple = self.receiver_ids.len() as u8;
        // self.local_cluster = Vec::new();
        println!("Robot {}: New Global contains {} robots", self.team.style(self.id.to_string()).bold(), self.not_received_simple);

        // Backup Cluster
        self.max_gold_receive = 0;
        self.max_gold_receive_coord = None;
        self.backup_cluster = Vec::new();

        // PAXOS
        self.consensus_coord = None;
        self.promised_message = None;
        self.max_id_seen = 0;
        self.max_piggyback_id_seen = 0;
        self.promise_count = 0;
        self.piggybacked = false;
        self.reached_majority = false;
        self.accept_count = 0;
        // self.majority = (self.local_cluster.len() / 2) as u8;
        self.majority = (self.not_received_simple / 2) as u8;
        self.increment = self.id as u32;
        self.send_pair_request = false;
        self.consensus_pair = None;
        self.pre_pickup_pair_id = None;
        self.accepted = false;

        // Direction Consensus
        self.sent_direction_request = false;
        self.received_direction = false;
        self.turned = false;

        // Resolve across clustser
        self.combined_pair_id = None;
        self.send_getout = false;
        self.override_target_gold = false;

        self.carrying_with_wrong_pair = false;

        // State Tracking
        self.current_state = RobotState::ClusterFinding;
        self.turn_direction = None;

        println!("{}", "RESET".bold());
    }

    pub fn scored(&mut self) {
        self.send(Message::new(
            self.id,
            MessageType::Done,
            self.id as u32,
            MessageContent::Coord(None, None),
        ), self.local_cluster.clone());
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

    fn get_receiver_ids(&self) -> Vec<char> {
        self.receiver_ids.clone()
    }


    pub fn is_carrying(&self) -> bool {
        self.is_carrying
    }

    pub fn was_carrying(&self) -> bool { self.was_carrying }
    pub fn get_pair_id(&self) -> Option<char> {
        self.pair_id
    }

}

// Decision logic 
impl Robot {
    pub fn make_decision(&mut self, manual: bool) -> Action {
        if self.is_carrying {
            self.was_carrying = true;
        }
        if self.not_received_simple == 0 && !self.send_pair_request {
            let mut rng = rand::rng();
            let pair_id = self.local_cluster.choose(&mut rng);
            if pair_id.is_some() {
                self.message_to_send = Some(Message::new(
                    self.id,
                    MessageType::PrepareRequest,
                    self.id as u32,
                    MessageContent::Pair(self.id, *pair_id.unwrap())),
                );
                self.send(self.message_to_send.unwrap(), self.local_cluster.clone());
                self.majority = (self.local_cluster.len() / 2) as u8;
                self.send_pair_request = true;
            }
        }
        self.paxos_receiver(self.receive());
        if manual {
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
        } else if !self.planned_actions.is_empty() {
            println!("{:?}", self.planned_actions);
            self.planned_actions.remove(0)
        } else {
            // Spam PICKUP
            if !self.is_carrying() && self.pre_pickup_pair_id.is_some() && self.turned {
                if self.current_coord == self.target_gold.unwrap() {
                    Action::PickUp
                } else {
                    self.received_begin = true;
                    self.receiver_ids = self.local_cluster.clone();
                    self.local_cluster.clear();
                    self.reset();
                    Action::Turn(Direction::Right)
                }
            } else if self.is_carrying() {
                if self.pre_pickup_pair_id.unwrap() == self.pair_id.unwrap() {
                    self.current_state = RobotState::MovingToDropBox;
                    self.plan_actions_to_move_to(self.deposit_box_coord);
                    Action::Turn(Direction::Up)
                } else {
                    self.carrying_with_wrong_pair = true;
                    Action::PickUp
                }
            } else {
                // Turn randomly
                match rand::random_range(1..5) {
                    1 => Turn(Direction::Left),
                    2 => Turn(Direction::Right),
                    3 => Turn(Direction::Up),
                    _ => Turn(Direction::Down),
                }
                // Act randomly
                // match rand::random_range(1..6) {
                //     1 => Turn(Direction::Left),
                //     2 => Turn(Direction::Right),
                //     3 => Turn(Direction::Down),
                //     4 => Turn(Direction::Up),
                //     _ => Action::Move,
                // }
                // match self.facing {
                //     Direction::Left => Turn(Direction::Right),
                //     Direction::Right => Turn(Direction::Left),
                //     Direction::Up => Turn(Direction::Down),
                //     Direction::Down => Turn(Direction::Up),
                // }
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

    pub fn get_latest_action(&self) -> Action {
        self.action_history.last().unwrap().clone()
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
        self.was_carrying = false;
        self.coord_history[self.turn - 1]
    }

    pub fn score_gold(&mut self) {
        match self.team {
            Team::Red => println!("{}{} has {}", "|".red(), self.id.to_string().red().bold(), "SCORED!".green().bold()),
            Team::Blue => println!("{}{} has {}", "|".blue(), self.id.to_string().blue().bold(), "SCORED!".green().bold()),
        }
        self.is_carrying = false;
        self.was_carrying = false;
    }
}

// Observation logic
impl Robot {

    pub fn observe(&mut self, grid: &mut Grid) {
        // let mut target = self.current_coord;
        for observable_cell in self.observable_cells.iter() {
            let observed_cell = grid.get_cell(*observable_cell).unwrap();
            if observed_cell.get_gold_amount().is_some() && !self.send_target {
                if observed_cell.get_gold_amount().unwrap() > self.max_gold_seen && !self.override_target_gold {
                    if self.old_target_gold.is_some() {
                        if observed_cell.coord != self.old_target_gold.unwrap() {
                            self.max_gold_seen = observed_cell.get_gold_amount().unwrap();
                            self.target_gold = Some(observed_cell.coord);
                            self.target_gold_amount = observed_cell.get_gold_amount().unwrap();
                            self.message_to_send = Some(Message::new(
                                self.id,
                                MessageType::Simple,
                                self.id as u32,
                                MessageContent::Coord(Some(observed_cell.coord), Some(observed_cell.get_gold_amount().unwrap())),
                            ));
                        }
                    } else {
                        self.max_gold_seen = observed_cell.get_gold_amount().unwrap();
                        self.target_gold = Some(observed_cell.coord);
                        self.target_gold_amount = observed_cell.get_gold_amount().unwrap();
                        self.message_to_send = Some(Message::new(
                            self.id,
                            MessageType::Simple,
                            self.id as u32,
                            MessageContent::Coord(Some(observed_cell.coord), Some(observed_cell.get_gold_amount().unwrap())),
                        ));

                    }
                }
            }
            // self.knowledge_base.entry(observed_cell.coord).or_insert(observed_cell);
            self.knowledge_base.insert(observed_cell.coord, observed_cell);

        }
        if !self.send_target {
            if self.target_gold.is_none() {
            } else {
                self.send(self.message_to_send.unwrap(), self.receiver_ids.clone());
                self.send_target = true;
            }
        }

        if self.consensus_coord.is_some() {
            // Reached target gold coord
            if self.current_coord == self.target_gold.unwrap() {
                self.current_state = RobotState::AtTarget;
                if !self.received_direction && !self.sent_direction_request {
                    if self.pre_pickup_pair_id.is_some() {
                        let propose_direction;
                        match rand::random_range(1..5) {
                            1 => propose_direction = Direction::Right,
                            2 => propose_direction = Direction::Left,
                            3 => propose_direction = Direction::Up,
                            4 => propose_direction = Direction::Down,
                            _ => propose_direction = Direction::Right,
                        }
                        self.send(Message::new(
                            self.id,
                            MessageType::Request,
                            self.id as u32,
                            MessageContent::TurnReq(propose_direction, self.target_gold.unwrap()),
                            // MessageContent::Direction(propose_direction),
                        ), vec![self.pre_pickup_pair_id.unwrap()]);
                        self.sent_direction_request = true;

                    }
                } else if self.turn_direction.is_some() {
                    self.planned_actions.push(Turn(self.turn_direction.unwrap()));
                    self.turn_direction = None;
                }

                // If see other robots at target gold, send GetOut
                if self.combined_pair_id.is_some() {
                    match self.team {
                        Team::Blue => {
                            if  self.knowledge_base.get(&self.target_gold.unwrap()).unwrap().blue_robots > 2 {
                                let (a, b) = self.consensus_pair.unwrap();
                                let filtered: Vec<char> = self.knowledge_base.get(&self.target_gold.unwrap()).unwrap().blue_robots_ids
                                  .iter()
                                  .filter(|&&c| c != a && c != b) // keep only chars that are not 'b' or 'd'
                                  .cloned() // since iter() gives &char, we clone to get Vec<char>
                                  .collect();
                                if !self.send_getout {
                                    // self.send_getout = true;
                                    self.send(Message::new(
                                        self.id,
                                        MessageType::GetOut,
                                        self.combined_pair_id.unwrap(),
                                        MessageContent::Coord(self.target_gold, Some(0u8)),
                                    ), filtered);

                                }

                            }
                        },
                        Team::Red => {
                            if  self.knowledge_base.get(&self.target_gold.unwrap()).unwrap().red_robots > 2 {
                                let (a, b) = self.consensus_pair.unwrap();
                                let filtered: Vec<char> = self.knowledge_base.get(&self.target_gold.unwrap()).unwrap().red_robots_ids
                                  .iter()
                                  .filter(|&&c| c != a && c != b) // keep only chars that are not 'b' or 'd'
                                  .cloned() // since iter() gives &char, we clone to get Vec<char>
                                  .collect();
                                if !self.send_getout {
                                    // self.send_getout = true;
                                    self.send(Message::new(
                                        self.id,
                                        MessageType::GetOut,
                                        self.combined_pair_id.unwrap(),
                                        MessageContent::Coord(self.target_gold, Some(0u8)),
                                    ), filtered);
                                }

                            }

                        }

                }

                }
            }
        }

        // Gold gone before reaching/picking
        if self.pre_pickup_pair_id.is_some() {
            if self.knowledge_base.get(&self.target_gold.unwrap()).is_some() {
                let current_target_cell = self.knowledge_base.get(&self.target_gold.unwrap()).unwrap();
                if current_target_cell.get_gold_amount().is_none() && self.is_second_check && !self.is_carrying {
                    // Send DONE and reset
                    self.received_begin = true;
                    self.receiver_ids = self.local_cluster.clone();
                    self.scored();
                    self.local_cluster.clear();
                    self.reset();
                    self.planned_actions.clear();
                    self.is_second_check = false;
                } else if current_target_cell.get_gold_amount().is_none() && !self.is_second_check {
                    self.is_second_check = true;
                }
            }
        }
        if self.logger_config.robot_kb {
            match self.team {
                Team::Red => println!("{}{:?} Robot {} Current KB: {:?}", "|".red(), self.team, self.id.to_string().red(), self.knowledge_base),
                Team::Blue => println!("{}{:?} Robot {} Current KB: {:?}", "|".blue(), self.team, self.id.to_string().blue(), self.knowledge_base),
            }
        }
    }
    pub fn observable_cells(&mut self, width: usize, height: usize) -> LinkedList<Coord> {
        let mut observable_cells: LinkedList::<Coord> = LinkedList::new();
        let mut current_coord = self.current_coord;
        observable_cells.push_back(current_coord);
        match self.facing {
            Direction::Left => {
                if current_coord.x == 0 {
                    self.observable_cells = observable_cells.clone();
                    return observable_cells;
                }
                current_coord.x -= 1
            },
            Direction::Right => {
                if current_coord.x == width - 1 {
                    self.observable_cells = observable_cells.clone();
                    return observable_cells;
                }
                current_coord.x += 1
            },
            Direction::Up => {
                if current_coord.y == height - 1 {
                    self.observable_cells = observable_cells.clone();
                    return observable_cells;
                }
                current_coord.y += 1
            },
            Direction::Down => {
                if current_coord.y == 0 {
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
                    if y + i < height {
                        observable_cells.push_back(Coord::new(x, y + i))
                    }
                },
                Direction::Up | Direction::Down => {
                    if x + i < width {
                        observable_cells.push_back(Coord::new(x + i, y))
                    }
                }
            }
        }
        match self.facing {
            Direction::Left => {
                if current_coord.y != 0 {
                    observable_cells.push_back(Coord::new(current_coord.x, current_coord.y - 1));
                }
                if current_coord.x == 0 {
                    self.observable_cells = observable_cells.clone();
                    return observable_cells;
                }
                current_coord.x -= 1
            },
            Direction::Right => {
                if current_coord.y != 0 {
                    observable_cells.push_back(Coord::new(current_coord.x, current_coord.y - 1));
                }
                if current_coord.x == width - 1 {
                    self.observable_cells = observable_cells.clone();
                    return observable_cells;
                }
                current_coord.x += 1
            },
            Direction::Up => {
                if current_coord.x != 0 {
                    observable_cells.push_back(Coord::new(current_coord.x - 1, current_coord.y));
                }
                if current_coord.y == height - 1 {
                    self.observable_cells = observable_cells.clone();
                    return observable_cells;
                }
                current_coord.y += 1
            },
            Direction::Down => {
                if current_coord.x != 0 {
                    observable_cells.push_back(Coord::new(current_coord.x - 1, current_coord.y));
                }
                if current_coord.y == 0 {
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
                    if y >= i {
                        observable_cells.push_back(Coord::new(x, y - i))}
                    }
                Direction::Up | Direction::Down => {
                    if x >= i {
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
                    if y + i < height {
                        observable_cells.push_back(Coord::new(x, y + i))
                    }
                },
            Direction::Up | Direction::Down => {
                if x + i < width {
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
            let mut random_timer_message = message;
            let mut rng = rand::rng();
            let timer = rng.random_range(0..=0);
            random_timer_message.timer = timer;
            message_board_guard.get_message_board().entry(receiver_id).or_default().send_messages(random_timer_message);
        }
    }

    fn receive(&self) -> Option<Message> {
        let mut message_board_guard = self.message_board.lock().unwrap();
        let mut message_to_return = None;
        if let Some(message_box) = message_board_guard.get_message_board().get_mut(&self.id) {
            message_to_return = message_box.retrieve_messages()
        }
        if self.logger_config.robot_message {
            match message_to_return {
                Some(message) => {
                    println!("Robot {} received {:?}", self.team.style(self.id.to_string()), message);
                },
                None => {
                    println!("Robot {} received None", self.team.style(self.id.to_string()));
                }
            }
        }
        message_to_return
    }

    fn set_consensus(&mut self, consensus: MessageContent) {
        match consensus {
            MessageContent::Coord(Some(coord), _) => {
                self.consensus_coord = Some(coord);
                println!("Robot {} has Consensus coord: {:?}", self.team.style(self.id.to_string()), self.consensus_coord);
            },
            MessageContent::Pair(a, b) => {
                self.consensus_pair = Some((a, b));
                self.received_begin = false;
                self.consensus_coord = self.target_gold;
                println!("Robot {} has Consensus pair: {:?}", self.team.style(self.id.to_string()), self.consensus_pair);
                // Self is chosen as designated pair
                // if (self.id == a || self.id == b) && self.planned_actions.is_empty() && self.target_gold.is_some() {
                if (self.id == a || self.id == b) && self.target_gold.is_some() {
                    self.planned_actions.clear();
                    if a as u32 > b as u32 {
                        self.combined_pair_id = Some(a as u32);
                    } else {
                        self.combined_pair_id = Some(b as u32);
                    }
                    if self.id == a {
                        self.pre_pickup_pair_id = Some(b);
                    } else {
                        self.pre_pickup_pair_id = Some(a);
                    }
                    self.plan_actions_to_move_to(self.target_gold.unwrap());
                    println!("Plan to move to {:?}: {:?}", self.target_gold.unwrap(), self.planned_actions);
                }
            },
            _ => {}
        }
    }

    fn paxos_receiver(&mut self, received_message: Option<Message>) {
        match received_message {
            Some(message) => {
                match message.msg_type {
                    MessageType::PrepareRequest => {
                        if self.current_state == RobotState::Paxos {
                            match self.promised_message {
                                Some(promised_message) => {
                                    if promised_message.id < message.id {
                                        println!("Robot {} Piggybacked", self.team.style(self.id.to_string()));
                                        self.promised_message = Some(Message::new(
                                            promised_message.sender_id,
                                            promised_message.msg_type,
                                            message.id,
                                            promised_message.message_content,
                                        ));
                                        println!("{:?}", self.promised_message);
                                        let piggyback_msg = Message::new(
                                            self.id,
                                            MessageType::PrepareResponse,
                                            promised_message.id,
                                            promised_message.message_content,
                                        );
                                        self.send(piggyback_msg, vec![message.sender_id]);
                                    } else {
                                        // let nack_msg = Message::new(
                                        //     self.id,
                                        //     MessageType::Nack,
                                        //     promised_message.id,
                                        //     promised_message.coord,
                                        // );
                                        // self.send(nack_msg, vec![message.sender_id]);
                                    }
                                },
                                None => {
                                    self.promised_message = Some(message);
                                    let promised = Message::new(
                                        self.id,
                                        MessageType::PrepareResponse,
                                        message.id,
                                        message.message_content,
                                    );
                                    self.send(promised, vec![message.sender_id]);
                                }
                            }
                        }
                    },
                    MessageType::AcceptRequest => {
                        if self.current_state == RobotState::Paxos {
                            match self.promised_message {
                                Some(promised_message) => {
                                    println!("Promised Message: {:?}", promised_message);
                                    println!("Received Message: {:?}", message);
                                    if promised_message.id <= message.id && !self.accepted {
                                        self.accepted = true;
                                        // self.set_consensus(message.message_content);
                                        self.promised_message = Some(message);
                                        let accepted_msg = Message::new(
                                            self.id,
                                            MessageType::Accepted,
                                            message.id,
                                            message.message_content,
                                        );
                                        self.send(accepted_msg, vec![message.sender_id]);
                                    } else {
                                        // let nack_msg = Message::new(
                                        //     self.id,
                                        //     MessageType::Nack,
                                        //     promised_message.id,
                                        //     promised_message.coord,
                                        // );
                                        // self.send(nack_msg, vec![message.sender_id]);
                                    }
                                },
                                None => {}
                            }
                        }
                    },
                    MessageType::PrepareResponse => {
                        if self.current_state == RobotState::Paxos {
                            self.promise_count += 1;
                            if message.id == self.message_to_send.unwrap().id && !self.piggybacked {
                                if self.promise_count > self.majority && !self.reached_majority {
                                    self.reached_majority = true;
                                    println!("Robot {} has received majority promises", self.team.style(self.id.to_string()));
                                    let message_to_send = self.message_to_send.unwrap();
                                    let accept_request_msg = Message::new(
                                        self.id,
                                        MessageType::AcceptRequest,
                                        message_to_send.id,
                                        message_to_send.message_content,
                                    );
                                    self.send(accept_request_msg, self.local_cluster.clone());
                                }
                            } else {
                                self.piggybacked = true;
                                // Update highset piggyback ID
                                if message.id > self.max_piggyback_id_seen {
                                    self.max_piggyback_id_seen = message.id;
                                    let message_to_send = self.message_to_send.unwrap();
                                    let new_message_to_send = Message::new(
                                        self.id,
                                        MessageType::AcceptRequest,
                                        message_to_send.id,
                                        message.message_content,
                                    );
                                    self.message_to_send = Some(new_message_to_send);
                                }
                                // Check majority
                                if self.promise_count > self.majority && !self.reached_majority {
                                    self.reached_majority = true;
                                    println!("Robot {} has received majority promises", self.team.style(self.id.to_string()));
                                    self.send(self.message_to_send.unwrap(), self.local_cluster.clone());
                                }
                            }
                        }
                    },
                    MessageType::Accepted => {
                        if self.current_state == RobotState::Paxos {
                            self.accept_count += 1;
                            if self.accept_count > self.majority {
                                self.set_consensus(message.message_content);
                                self.promised_message = Some(message);
                                self.send(Message::new(
                                    self.id,
                                    MessageType::Confirm,
                                    self.id as u32,
                                    message.message_content,
                                ), self.local_cluster.clone());
                                self.current_state = RobotState::MovingToTarget;
                            }
                        }
                    },
                    MessageType::Confirm => {
                        if self.current_state == RobotState::Paxos {
                            self.set_consensus(message.message_content);
                            self.current_state = RobotState::MovingToTarget;
                        }
                    }
                    MessageType::Nack => {
                        if self.current_state == RobotState::Paxos {
                            self.max_id_seen = message.id;
                            let Message { message_content, .. } = self.message_to_send.unwrap();
                            let new_message_to_send = Message::new(
                                self.id,
                                MessageType::PrepareRequest,
                                self.max_id_seen + self.increment,
                                message_content,
                            );
                            self.message_to_send = Some(new_message_to_send);
                            self.send(new_message_to_send, self.local_cluster.clone());
                        }
                    },
                    MessageType::Simple => {
                            if !self.received_begin {
                                self.received_begin = true;
                                self.receiver_ids = self.local_cluster.clone();
                                self.local_cluster.clear();
                                self.reset();
                            }
                            if self.not_received_simple > 0 {
                                self.not_received_simple -= 1;
                                if self.not_received_simple == 0 {
                                    self.current_state = RobotState::Paxos;
                                }
                                if self.target_gold.is_some() {
                                    match message.message_content {
                                        MessageContent::Coord(Some(coord), Some(gold_amount)) => {
                                            let list = self.clusters.entry((coord, gold_amount)).or_insert(vec![]);
                                            list.push(message.sender_id);
                                            if coord == self.target_gold.unwrap() {
                                                self.local_cluster.push(message.sender_id);
                                            }
                                            if gold_amount > self.max_gold_receive {
                                                self.backup_cluster.clear();
                                                self.max_gold_receive = gold_amount;
                                                self.max_gold_receive_coord = Some(coord);
                                            }
                                            match self.max_gold_receive_coord {
                                                Some(max_gold_coord) => {
                                                    if coord == max_gold_coord {
                                                        self.backup_cluster.push(message.sender_id);
                                                    }
                                                },
                                                _ => {}
                                            }
                                        },
                                        _ => {}
                                    }
                                } else {
                                    match message.message_content {
                                        MessageContent::Coord(Some(coord), Some(gold_amount)) => {
                                            self.override_target_gold = true;
                                            self.target_gold = Some(coord);
                                            self.max_gold_seen = gold_amount;
                                            self.target_gold_amount = gold_amount;
                                            self.message_to_send = Some(Message::new(
                                                self.id,
                                                MessageType::Simple,
                                                self.id as u32,
                                                MessageContent::Coord(Some(coord), Some(gold_amount)),
                                            ));
                                            let list = self.clusters.entry((coord, gold_amount)).or_insert(vec![]);
                                            list.push(message.sender_id);
                                            if coord == self.target_gold.unwrap() {
                                                self.local_cluster.push(message.sender_id);
                                            }
                                            if gold_amount > self.max_gold_receive {
                                                self.backup_cluster.clear();
                                                self.max_gold_receive = gold_amount;
                                                self.max_gold_receive_coord = Some(coord);
                                            }
                                            match self.max_gold_receive_coord {
                                                Some(max_gold_coord) => {
                                                    if coord == max_gold_coord {
                                                        self.backup_cluster.push(message.sender_id);
                                                    }
                                                },
                                                _ => {}
                                            }
                                        },
                                        _ => {}
                                    }
                                }
                                if self.not_received_simple == 0 && self.local_cluster.is_empty() {
                                    let mut singles = Vec::new();
                                    let mut max_key: Option<u8> = Some(self.target_gold_amount);
                                    let mut max_coord: Option<Coord> = self.target_gold;

                                    for (&(coord, gold_amount), v) in &self.clusters {
                                        if v.len() == 1 {
                                            singles.push(v[0]);

                                            // track max key
                                            match max_key {
                                                Some(current) => {
                                                    if gold_amount == current {
                                                        match max_coord {
                                                            Some(current_coord) => {
                                                                if coord.priority(current_coord) {
                                                                    max_coord = Some(coord);
                                                                    max_key = Some(current);
                                                                }
                                                            },
                                                            _ => {}
                                                        }
                                                    } else if gold_amount > current {
                                                        max_coord = Some(coord);
                                                        max_key = Some(current);
                                                    }
                                                },
                                                _ => {
                                                    max_coord = Some(coord);
                                                    max_key = Some(gold_amount);
                                                }
                                            }
                                            max_key = Some(match max_key {
                                                Some(current) if current > gold_amount => current,
                                                _ => gold_amount,
                                            });
                                        }
                                    }

                                    self.local_cluster = singles;
                                    self.target_gold = max_coord;
                                    // self.consensus_coord = max_coord;
                                    self.current_state = RobotState::Paxos;
                                }
                            }
                    },
                    MessageType::Request => {
                        if self.target_gold.is_some() && !self.turned {
                            match message.message_content {
                                MessageContent::TurnReq(direction, coord) => {
                                    if coord == self.target_gold.unwrap() {
                                        if self.id < message.sender_id || self.current_coord != self.target_gold.unwrap() {
                                            self.send(Message::new(
                                                self.id,
                                                MessageType::Ack,
                                                self.id as u32,
                                                message.message_content,
                                            ), vec![message.sender_id]);
                                            self.turn_direction = Some(direction);
                                            self.planned_actions.push(Turn(direction));
                                            self.received_direction = true;
                                            self.turned = true;
                                        }
                                    }
                                },
                                _ => {}
                            }
                        }
                    },
                    MessageType::Ack => {
                        if self.target_gold.is_some() {
                            match message.message_content {
                                MessageContent::TurnReq(direction, coord) => {
                                    if coord == self.target_gold.unwrap() {
                                        if self.current_coord == self.target_gold.unwrap() {
                                            self.planned_actions.push(Turn(direction));
                                            self.turned = true;
                                        }
                                    }
                                },
                                _ => {}
                            }
                        }
                    },
                    MessageType::Done => {
                        if !self.received_begin {
                            self.received_begin = true;
                            self.receiver_ids = self.local_cluster.clone();
                            self.local_cluster.clear();
                            self.reset();
                        }
                    },
                    MessageType::GetOut => {
                        match self.combined_pair_id {
                            Some(combined_pair_id) => {
                                match message.message_content {
                                    MessageContent::Coord(Some(get_out_coord), _) => {
                                        if self.current_coord == get_out_coord && message.id > combined_pair_id && self.current_coord == self.target_gold.unwrap() && (!self.is_carrying || self.carrying_with_wrong_pair) {
                                            self.old_target_gold = self.target_gold;
                                            self.received_begin = true;
                                            self.receiver_ids = self.local_cluster.clone();
                                            self.scored();
                                            self.local_cluster.clear();
                                            self.reset();
                                            self.planned_actions.clear();
                                            self.plan_actions_to_move_to(self.deposit_box_coord);
                                            // self.planned_actions.push(Action::Turn(Direction::Left));
                                            // self.planned_actions.push(Action::Move);
                                            // self.planned_actions.push(Action::Turn(Direction::Up));
                                            // self.planned_actions.push(Action::Move);
                                            // self.planned_actions.push(Action::Turn(Direction::Right));
                                            // self.planned_actions.push(Action::Move);
                                        } else if self.planned_actions.is_empty() && self.current_state != RobotState::AtTarget {
                                            self.plan_actions_to_move_to(self.deposit_box_coord);
                                        }
                                    },
                                    _ => {
                                        if self.planned_actions.is_empty() {
                                            self.plan_actions_to_move_to(self.deposit_box_coord);
                                        }
                                    }
                                }
                            },
                            None => {}
                        }

                    }
                }
            },
            None => ()
        }
    }
}


// Move Planning
impl Robot {
    pub fn plan_actions_to_move_to(&mut self, target: Coord) {
        let current = self.current_coord;
        let travel_x = target.x as i32 - current.x as i32;
        let travel_y = target.y as i32 - current.y as i32;

        let at_x = travel_x == 0;
        let at_y = travel_y == 0;

        let facing_x;
        let facing_y;
        if travel_x > 0 {
            facing_x = Direction::Right;
        } else {
            facing_x = Direction::Left;
        }
        if travel_y > 0 {
            facing_y = Direction::Up;
        } else {
            facing_y = Direction::Down;
        }

        if self.facing == facing_x {
            if !at_x {
                self.plan_move(travel_x.abs());
            }
            if !at_y {
                self.planned_actions.push(Action::Turn(facing_y));
                self.plan_move(travel_y.abs());
            }
        } else if self.facing == facing_y {
            if !at_y {
                self.plan_move(travel_y.abs());
            }
            if !at_x {
                self.planned_actions.push(Action::Turn(facing_x));
                self.plan_move(travel_x.abs());
            }
        } else {
            if !at_x {
                self.planned_actions.push(Action::Turn(facing_x));
                self.plan_move(travel_x.abs());
            }
            if !at_y {
                self.planned_actions.push(Action::Turn(facing_y));
                self.plan_move(travel_y.abs());
            }
        }

    }

    fn plan_move(&mut self, distance: i32) {
        for _ in 0..distance {
            self.planned_actions.push(Action::Move);
        }
    }
}

// Print functions
impl Debug for Robot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.team {
            Team::Red => {
                write!(f, "{}({:?}) is at {:?} facing {:?} - ", self.id.to_string().red(), self.current_state, self.current_coord, self.facing)?;
                write!(f, "Consensus coord: {:?} - ", self.consensus_coord)?;
                write!(f, "Consensus pair: {:?} - ", self.consensus_pair)?;
                write!(f, "Target gold: {:?} - ", self.target_gold)?;
                write!(f, "Local cluster: {:?}", self.local_cluster)?;
                if self.is_carrying {
                    write!(f, " is {} with {}", "CARRYING GOLD".yellow().bold(), self.pair_id.unwrap().to_string().red().dimmed())
                } else {
                    write!(f, "")
                }
            },
            Team::Blue => {
                write!(f, "{}({:?}) is at {:?} facing {:?} - ", self.id.to_string().blue(), self.current_state, self.current_coord, self.facing)?;
                write!(f, "Consensus coord: {:?} - ", self.consensus_coord)?;
                write!(f, "Consensus pair: {:?} - ", self.consensus_pair)?;
                write!(f, "Target gold: {:?} - ", self.target_gold)?;
                write!(f, "Local cluster: {:?}", self.local_cluster)?;
                if self.is_carrying {
                    write!(f, " is {} with {}", "CARRYING GOLD".yellow().bold(), self.pair_id.unwrap().to_string().blue().dimmed())
                } else {
                    write!(f, "")
                }
            },
        }
    }
}

// Utility Functions
fn make_vec(n: u8, x: char, team: Team) -> Vec<char> {
    match team {
        Team::Blue => {
            let mut chars: Vec<char> = (0..n)
                .map(|i| (b'a' + i) as char) // convert u8 -> char
                .collect();

            chars.retain(|&c| c != x); // remove the special char
            chars
        },
        Team::Red => {
            let mut chars: Vec<char> = (0..n)
                .map(|i| (b'A' + i) as char) // convert u8 -> char
                .collect();

            chars.retain(|&c| c != x); // remove the special char
            chars
        }
    }
}
