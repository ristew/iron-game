#![feature(drain_filter)]

use ggez::{
    conf::{WindowMode, WindowSetup},
    event, ContextBuilder,
};
#[macro_use]
extern crate iron_derive;

pub mod commands;
pub mod events;
pub mod game;
pub mod math;
pub mod pops;
pub mod probability;
pub mod render;
pub mod storage;
pub mod ui;
pub mod world;
pub mod worldgen;

pub use commands::*;
pub use events::*;
pub use game::*;
pub use math::*;
pub use pops::*;
pub use probability::*;
pub use render::*;
pub use std::cell::RefCell;
pub use std::rc::Rc;
pub use storage::*;
pub use ui::*;
pub use world::*;
pub use worldgen::*;

fn main() {
    let cb = ContextBuilder::new("iron-age", "ristew")
        .window_setup(WindowSetup::default().vsync(false).title("iron-age"))
        .window_mode(WindowMode::default().dimensions(1200.0, 800.0));
    let (mut ctx, evt_loop) = cb.build().unwrap();
    let game = game::MainState::new(&mut ctx);
    event::run(ctx, evt_loop, game);
}
