use crate::util::Coord;

#[derive(Copy, Clone)]
pub enum Team {
    Red,
    Blue,
}

pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

pub struct Robot {
    id: char,
    team: Team,
    current_pos: Coord,
    facing: Direction,
}

impl Robot {
    
    pub fn new(id: char, team: Team, current_pos: Coord, facing: Direction) -> Self {
        Robot { id, team, current_pos, facing }
    }
    
    pub fn get_team(&self) -> Team {
        self.team
    }

    pub fn get_id(&self) -> char {
        self.id
    }
}

