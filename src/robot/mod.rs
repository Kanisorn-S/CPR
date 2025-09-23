pub mod manager;

use std::collections::{LinkedList, HashMap};
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use std::io;
use crate::util::Coord;
use colored::{ColoredString, Colorize};
use crate::communication::message::{Message, MessageBoard, MessageContent, MessageType};
use crate::config::logger::LoggerConfig;
use crate::environment::cell::Cell;
use crate::environment::grid::Grid;
use crate::config::Config;

use rand::seq::IndexedRandom;


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
#[derive(Copy, Clone, PartialEq)]
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

    // Perception
    observable_cells: LinkedList<Coord>,
    knowledge_base: HashMap<Coord, Cell>,

    // Communication
    message_board: Arc<Mutex<MessageBoard>>,
    consensus_coord: Option<Coord>,
    promised_message: Option<Message>,
    max_id_seen: u32,
    max_piggyback_id_seen: u32,
    message_to_send: Option<Message>,
    receiver_ids: Vec<char>,
    promise_count: u8,
    piggybacked: bool,
    reached_majority: bool,
    accept_count: u8,
    majority: u8,
    increment: u32,
    send_pair_request: bool,
    consensus_pair: Option<(char, char)>,

    target_gold: Option<Coord>,
    max_gold_seen: u8,
    send_target: bool,
    local_cluster: Vec<char>,
    not_received_simple: u8,

    // Move Planning
    planned_actions: Vec<Action>,
    
    // Configurations
    logger_config: LoggerConfig,
}

// Constructors and getters
impl Robot {
    pub fn new(id: char, team: Team, current_coord:Coord, facing: Direction, message_board: Arc<Mutex<MessageBoard>>) -> Self {
        let mut coord_history: Vec<Coord> = Vec::new();
        let Config { n_robots, .. } = Config::new();
        coord_history.push(current_coord);
        Robot {
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
            observable_cells: LinkedList::new(),
            knowledge_base: HashMap::new(),
            message_board,
            consensus_coord: None,
            promised_message: None,
            max_id_seen: 0,
            max_piggyback_id_seen: 0,
            piggybacked: false,
            message_to_send: Some(Message::new(
                id,
                MessageType::PrepareRequest,
                id as u32,
                MessageContent::Coord(Some(current_coord)),
            )),
            receiver_ids: make_vec(n_robots, id, team),
            promise_count: 0,
            reached_majority: false,
            accept_count: 0,
            majority: n_robots / 2,
            increment: id as u32,
            send_pair_request: false,
            consensus_pair: None,
            target_gold: None,
            max_gold_seen: 0,
            send_target: false,
            local_cluster: Vec::new(),
            not_received_simple: n_robots - 1,
            planned_actions: Vec::new(),
            logger_config: LoggerConfig::new(),
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
        if (self.not_received_simple == 0 && !self.send_pair_request) {
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
        } else if !self.planned_actions.is_empty() {
            self.planned_actions.remove(0)
        } else {
            match rand::random_range(1..5) {
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
        let mut target = self.current_coord;
        for observable_cell in self.observable_cells.iter() {
            let observed_cell = grid.get_cell(*observable_cell).unwrap();
            if observed_cell.get_gold_amount().is_some() && !self.send_target {
                if observed_cell.get_gold_amount().unwrap() > self.max_gold_seen {
                    self.max_gold_seen = observed_cell.get_gold_amount().unwrap();
                    self.target_gold = Some(observed_cell.coord);
                    self.message_to_send = Some(Message::new(
                        self.id,
                        MessageType::Simple,
                        self.id as u32,
                        MessageContent::Coord(Some(observed_cell.coord)),
                    ));
                }
            }
            self.knowledge_base.entry(observed_cell.coord).or_insert(observed_cell);
            target = observed_cell.coord;
        }
        if (self.turn == 0) {
            self.plan_actions_to_move_to(target);
            println!("Plan to move to {:?}: {:?}", target, self.planned_actions);
        }
        if !self.send_target {
            if self.target_gold.is_none() {
                let message = Message::new(
                    self.id,
                    MessageType::Simple,
                    self.id as u32,
                    MessageContent::Coord(None),
                );
                self.send(message, self.receiver_ids.clone());
            } else {
                self.send(self.message_to_send.unwrap(), self.receiver_ids.clone());
            }
            self.send_target = true;
        }
        if (self.logger_config.robot_kb) {
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
            message_board_guard.get_message_board().entry(receiver_id).or_default().send_messages(message);
        }
    }

    fn receive(&self) -> Option<Message> {
        let mut message_board_guard = self.message_board.lock().unwrap();
        let mut message_to_return = None;
        if let Some(message_box) = message_board_guard.get_message_board().get_mut(&self.id) {
            message_to_return = message_box.retrieve_messages()
        }
        if (self.logger_config.robot_message) {
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
            MessageContent::Coord(Some(coord)) => {
                self.consensus_coord = Some(coord);
                println!("Robot {} has Consensus coord: {:?}", self.team.style(self.id.to_string()), self.consensus_coord);
            },
            MessageContent::Pair(a, b) => {
                self.consensus_pair = Some((a, b));
                println!("Robot {} has Consensus pair: {:?}", self.team.style(self.id.to_string()), self.consensus_pair);
                self.set_consensus(MessageContent::Coord(Some(self.target_gold.unwrap())));
            },
            _ => {}
        }
    }

    fn paxos_receiver(&mut self, received_message: Option<Message>) {
        match received_message {
            Some(message) => {
                match message.msg_type {
                    MessageType::PrepareRequest => {
                        match self.promised_message {
                            Some(promised_message) => {
                                if (promised_message.id < message.id) {
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
                    },
                    MessageType::AcceptRequest => {
                        match self.promised_message {
                            Some(promised_message) => {
                                println!("Promised Message: {:?}", promised_message);
                                println!("Received Message: {:?}", message);
                                if (promised_message.id <= message.id) {
                                    self.set_consensus(message.message_content);
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
                    },
                    MessageType::PrepareResponse => {
                        self.promise_count += 1;
                        if (message.id == self.message_to_send.unwrap().id && !self.piggybacked) {
                            if (self.promise_count > self.majority && !self.reached_majority) {
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
                            if (message.id > self.max_piggyback_id_seen) {
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
                            if (self.promise_count > self.majority && !self.reached_majority) {
                                self.reached_majority = true;
                                println!("Robot {} has received majority promises", self.team.style(self.id.to_string()));
                                self.send(self.message_to_send.unwrap(), self.local_cluster.clone());
                            }
                        }
                    },
                    MessageType::Accepted => {
                        self.accept_count += 1;
                        if (self.accept_count > self.majority) {
                            self.set_consensus(message.message_content);
                            self.promised_message = Some(message);
                        }
                    },
                    MessageType::Nack => {
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
                    },
                    MessageType::Simple => {
                        if self.not_received_simple > 0 {
                            self.not_received_simple -= 1;
                            if self.target_gold.is_some() {
                                match message.message_content {
                                    MessageContent::Coord(Some(coord)) => {
                                        if coord == self.target_gold.unwrap() {
                                            self.local_cluster.push(message.sender_id);
                                        }
                                    },
                                    _ => {}
                                }
                            }
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

        if (self.facing == facing_x) {
            if !at_x {
                self.plan_move(travel_x.abs());
            }
            if !at_y {
                self.planned_actions.push(Action::Turn(facing_y));
                self.plan_move(travel_y.abs());
            }
        } else if (self.facing == facing_y) {
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
                write!(f, "{} is at {:?} facing {:?} - ", self.id.to_string().red(), self.current_coord, self.facing)?;
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
                write!(f, "{} is at {:?} facing {:?} - ", self.id.to_string().blue(), self.current_coord, self.facing)?;
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
