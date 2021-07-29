#![feature(drain_filter)]
#[macro_use]
extern crate iron_derive;

pub mod game;
pub mod commands;
pub mod world;
pub mod probability;
pub mod storage;

fn main() {
    game::game_loop();
}
