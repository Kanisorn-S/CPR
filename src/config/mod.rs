pub mod logger;

// General Configurations
const WIDTH: usize = 5;
const HEIGHT: usize = 5;
const P_GOLD: f64 = 0.8;
const MAX_GOLD: u8 = 5;
const N_ROBOTS: u8 = 10;
const TURNS: u32 = 400;
const MANUAL: bool = false;
pub struct Config {
    pub width: usize,
    pub height: usize,
    pub p_gold: f64,
    pub max_gold: u8,
    pub n_robots: u8,
    pub turns: u32,
    pub manual: bool,
}

impl Config {
    pub fn new() -> Self {
        Config {
            width: WIDTH,
            height: HEIGHT,
            p_gold: P_GOLD,
            max_gold: MAX_GOLD,
            n_robots: N_ROBOTS,
            turns: TURNS,
            manual: MANUAL,
        }
    }
}
