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
pub mod ui;

pub use game::*;
pub use commands::*;
pub use world::*;
pub use probability::*;
pub use storage::*;
pub use render::*;
pub use pops::*;
pub use math::*;
pub use events::*;
pub use ui::*;
pub use std::rc::Rc;
pub use std::cell::RefCell;

fn main() {
    let cb = ContextBuilder::new("iron-age", "ristew")
        .window_setup(WindowSetup::default().vsync(false).title("iron-age"))
        .window_mode(WindowMode::default().dimensions(1200.0, 800.0));
    let (mut ctx, evt_loop) = cb.build().unwrap();
    let game = game::MainState::new(&mut ctx);
    event::run(ctx, evt_loop, game);
}
