use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use colored::Colorize;
use crate::util::Coord;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum MessageType {
  PrepareRequest,
  PrepareResponse,
  AcceptRequest,
  Accepted,
  Nack
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum MessageContent {
  Coord(Coord),
  Pair(char, char),
}

#[derive(PartialEq, Hash, Eq, Clone, Copy)]
pub struct Message {
  pub sender_id: char,
  pub msg_type: MessageType,
  pub id: u32,
  pub message_content: MessageContent,
}

impl Message {
  pub fn new(sender_id: char, msg_type: MessageType, id: u32, message_content: MessageContent) -> Message {
    Self {
      sender_id,
      msg_type,
      id,
      message_content,
    }
  }
}

#[derive(Default)]
pub struct MessageBox {
  current_messages: HashSet<Message>,
  new_messages: HashSet<Message>,
}

impl MessageBox {
  pub fn new() -> MessageBox {
    Self {
      current_messages: HashSet::new(),
      new_messages: HashSet::new(),
    }
  }

  pub fn update_messages(&mut self) {
    self.current_messages.extend(self.new_messages.drain());
  }

  pub fn send_messages(&mut self, message: Message) {
    self.new_messages.insert(message);
  }

  pub fn retrieve_messages(&mut self) -> Option<Message> {
    let random_message = self.current_messages.iter().next();
    let mut return_message = None;
    let mut message_available = false;
    match random_message {
      Some(message) => {
        return_message = Some(message.clone());
        message_available = true;
      },
      None => {}
    }
    if (message_available) {
      self.current_messages.remove(&return_message.unwrap());
    }
    return_message
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
      MessageContent::Coord(coord) => {
        write!(f, "{:?}", coord)
      }
    }
  }
}


impl Debug for Message {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}: {:?} - {:?} from {}", self.id, self.msg_type, self.message_content, self.sender_id)
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