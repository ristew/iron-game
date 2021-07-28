#![feature(drain_filter)]
#[macro_use]
extern crate iron_derive;

pub mod game;
mod probability;

fn main() {
    game::game_loop();
}
