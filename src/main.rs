#[allow(warnings)]
extern crate CPR;

use CPR::environment::{World};
use CPR::config::Config;
use colored::Colorize;
use CPR::config::logger::LoggerConfig;


fn main() {
    let Config {
        width,
        height,
        p_gold,
        max_gold,
        n_robots,
        manual,
        turns,
    } = Config::new();
    let mut world = World::new(width, height, p_gold, max_gold, n_robots, manual);
    let LoggerConfig {
        current_grid,
        robot_status,
        ..
    } = LoggerConfig::new();
    println!("{}", "Initial Grid".bold());
    world.print_grid();
    println!("{}", "-".repeat(100).bold());
    for i in 0..turns {
        println!("{} {}", "TURN".bold(), i.to_string().bold());
        if current_grid {
            println!("{}", "Current Grid".bold());
            world.print_grid();
        }
        if robot_status {
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