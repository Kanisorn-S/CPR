extern crate CPR;

use CPR::environment::{World};
use colored::Colorize;

const WIDTH: usize = 3;
const HEIGHT: usize = 3;
const P_GOLD: f64 = 0.5;
const MAX_GOLD: u8 = 5;
const N_ROBOTS: u8 = 2;
const TURNS: u8 = 200;
const MANUAL: bool = true;

fn main() {
    let mut world = World::new(WIDTH, HEIGHT, P_GOLD, MAX_GOLD, N_ROBOTS, MANUAL);
    println!("{}", "Initial Grid".bold());
    world.print_grid();
    println!("{}", "-".repeat(100).bold());
    for i in 0..TURNS {
        println!("{} {}", "TURN".bold(), i.to_string().bold());
        println!("{}", "Current Grid".bold());
        world.print_grid();
        println!("\n{}", "Current Robot Status".bold());
        world.print_robots();
        println!();
        world.next_turn();
        println!("{}", "-".repeat(100).bold());
    }
    println!("{}", "Final Grid".bold());
    world.print_grid();
}