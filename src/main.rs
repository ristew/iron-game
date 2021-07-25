#[macro_use]
extern crate iron_derive;

pub mod game;

fn main() {
    game::game_loop();
}
