#![feature(drain_filter)]

use ggez::{ContextBuilder, conf::{WindowMode, WindowSetup}, event};
#[macro_use]
extern crate iron_derive;

pub mod game;
pub mod commands;
pub mod world;
pub mod probability;
pub mod storage;
pub mod render;
pub mod pops;
pub mod math;
pub mod events;

pub use game::*;
pub use commands::*;
pub use world::*;
pub use probability::*;
pub use storage::*;
pub use render::*;
pub use pops::*;
pub use math::*;
pub use events::*;
pub use std::rc::Rc;
pub use std::cell::RefCell;

fn main() {
    let cb = ContextBuilder::new("iron-age", "ristew")
        .window_setup(WindowSetup::default().vsync(false))
        .window_mode(WindowMode::default().dimensions(640.0, 480.0));
    let (mut ctx, evt_loop) = cb.build().unwrap();
    let game = game::MainState::new(&mut ctx);
    event::run(ctx, evt_loop, game);
}
