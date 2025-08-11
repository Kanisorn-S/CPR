extern crate CPR;

use CPR::environment::{World};

const WIDTH: u8 = 3;
const HEIGHT: u8 = 3;
const P_GOLD: f64 = 1.0;
const MAX_GOLD: u8 = 5;
const N_ROBOTS: u8 = 2;

fn main() {
    let mut world = World::new(WIDTH, HEIGHT, P_GOLD, MAX_GOLD, N_ROBOTS);
    world.print_grid();
    world.make_decisions_and_take_actions();
    world.print_pickup_check();
    world.print_grid();

}