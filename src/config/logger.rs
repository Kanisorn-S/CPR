const CURRENT_GRID: bool = false;
const ROBOT_STATUS: bool = false;
const ROBOT_OBSERVATION: bool = false;
const ROBOT_DECISION: bool = false;
const MESSAGE_BOARD: bool = false;
const ROBOT_KB: bool = false;

pub struct LoggerConfig {
    pub current_grid: bool,
    pub robot_status: bool,
    pub robot_observation: bool,
    pub robot_decision: bool,
    pub message_board: bool,
    pub robot_kb: bool,
}

impl LoggerConfig {
    pub fn new() -> LoggerConfig {
        Self {
            current_grid: CURRENT_GRID,
            robot_status: ROBOT_STATUS,
            robot_observation: ROBOT_OBSERVATION,
            robot_decision: ROBOT_DECISION,
            message_board: MESSAGE_BOARD,
            robot_kb: ROBOT_KB,
        }
    }
}