use std::collections::{HashMap};
use std::fmt::{Debug, Display, Formatter};
use colored::Colorize;
use rand::Rng;
use crate::robot::Direction;
use crate::util::Coord;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum MessageType {
  PrepareRequest,
  PrepareResponse,
  AcceptRequest,
  Accepted,
  Confirm,
  Nack,
  Simple,
  Request,
  Ack,
  Done,
  GetOut,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum MessageContent {
  Coord(Option<Coord>, Option<u8>),
  Pair(char, char),
  Direction(Direction),
  TurnReq(Direction, Coord),
}

#[derive(PartialEq, Hash, Eq, Clone, Copy)]
pub struct Message {
  pub sender_id: char,
  pub msg_type: MessageType,
  pub id: u32,
  pub message_content: MessageContent,
  pub timer: u8,
}

impl Message {
  pub fn new(sender_id: char, msg_type: MessageType, id: u32, message_content: MessageContent) -> Message {
    let mut rng = rand::rng();
    let timer = rng.random_range(0..=3);
    Self {
      sender_id,
      msg_type,
      id,
      message_content,
      timer,
    }
  }
}

#[derive(Default)]
pub struct MessageBox {
  current_messages: Vec<Message>,
  new_messages: Vec<Message>,
}

impl MessageBox {
  pub fn new() -> MessageBox {
    Self {
      current_messages: Vec::new(),
      new_messages: Vec::new(),
    }
  }

  pub fn update_messages(&mut self) {
    self.current_messages.extend(self.new_messages.drain(..));
  }

  pub fn send_messages(&mut self, message: Message) {
    self.new_messages.push(message);
  }

  pub fn retrieve_messages(&mut self) -> Option<Message> {
    if !self.current_messages.is_empty() {
      let mut rng = rand::rng();
      let random_index = rng.random_range(0..self.current_messages.len());
      let random_message = self.current_messages.get_mut(random_index);
      let mut return_message = None;
      let mut message_available = false;
      match random_message {
        Some(message) => {
          if message.timer == 0 {
            return_message = Some(message.clone());
            message_available = true;
          } else {
            message.timer -= 1;
          }
        },
        None => {}
      }
      if message_available {
        self.current_messages.remove(random_index);
      }
      return_message
    } else {
      None
    }
  }
}

pub struct MessageBoard {
  message_board: HashMap<char, MessageBox>
}

impl MessageBoard {
  pub fn new() -> MessageBoard {
    Self {
      message_board: HashMap::new(),
    }
  }

  pub fn insert(&mut self, id: char, message_box: MessageBox) {
    self.message_board.insert(id, message_box);
  }

  pub fn get_message_board(&mut self) -> &mut HashMap<char, MessageBox> {
    &mut self.message_board
  }

  pub fn update(&mut self) {
    for message_box in self.message_board.values_mut() {
      message_box.update_messages();
    }
  }
}

// Print Functions
impl Debug for MessageContent {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      MessageContent::Pair(a, b) => {
        write!(f, "Pair {} {}", a, b)
      },
      MessageContent::Coord(coord, gold_amount) => {
        write!(f, "{:?} has {:?} golds", coord, gold_amount)
      },
      MessageContent::Direction(direction) => {
        write!(f, "{:?}", direction)
      },
      MessageContent::TurnReq(direction, coord) => {
        write!(f, "{:?} has {:?} coords", direction, coord)
      }
    }
  }
}


impl Debug for Message {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}: {:?} - {:?} from {} ({})", self.id, self.msg_type, self.message_content, self.sender_id, self.timer)
  }
}

impl Debug for MessageBox {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}: {:?} {} ", "CURRENT".yellow(), self.current_messages, "-".bold())?;
    write!(f, "{}: {:?}", "NEW".green(), self.new_messages)
  }
}

impl Display for MessageBox {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self.current_messages)
  }
}

impl Debug for MessageBoard {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    for (id, message_box) in self.message_board.iter() {
      write!(f, "  {}: {:?}\n", id, message_box)?;
    }
    write!(f, "")
  }
}

impl Display for MessageBoard {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    for (id, message_box) in self.message_board.iter() {
      write!(f, "  {}: {}\n", id, message_box)?;
    }
    write!(f, "")
  }
  
}