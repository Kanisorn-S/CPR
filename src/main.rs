extern crate CPR;

use CPR::environment::{World};

const WIDTH: u8 = 5;
const HEIGHT: u8 = 5;
const P_GOLD: f64 = 0.5;
const MAX_GOLD: u8 = 5;
const N_ROBOTS: u8 = 2;

fn main() {
    let mut world = World::new(WIDTH, HEIGHT, P_GOLD, MAX_GOLD, N_ROBOTS);
    world.print_grid();
    world.make_decisions_and_take_actions();
    world.print_grid();
}