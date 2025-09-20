extern crate CPR;

use CPR::environment::{World};
use colored::Colorize;
use CPR::config::logger::LoggerConfig;

const WIDTH: usize = 3;
const HEIGHT: usize = 3;
const P_GOLD: f64 = 0.5;
const MAX_GOLD: u8 = 5;
const N_ROBOTS: u8 = 2;
const TURNS: u8 = 2;
const MANUAL: bool = false;

fn main() {
    let mut world = World::new(WIDTH, HEIGHT, P_GOLD, MAX_GOLD, N_ROBOTS, MANUAL);
    let LoggerConfig {
        current_grid,
        robot_status,
        ..
    } = LoggerConfig::new();
    println!("{}", "Initial Grid".bold());
    world.print_grid();
    println!("{}", "-".repeat(100).bold());
    for i in 0..TURNS {
        println!("{} {}", "TURN".bold(), i.to_string().bold());
        if (current_grid) {
            println!("{}", "Current Grid".bold());
            world.print_grid();
        }
        if (robot_status) {
            println!("\n{}", "Current Robot Status".bold());
            world.print_robots();
        }
        println!();
        world.next_turn();
        println!("{}", "-".repeat(100).bold());
    }
    println!("{}", "Final Grid".bold());
    world.print_grid();
}