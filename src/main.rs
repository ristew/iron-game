#![feature(drain_filter)]
#![feature(trace_macros)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use ggez::{
    conf::{WindowMode, WindowSetup},
    event, ContextBuilder,
};
#[macro_use]
extern crate iron_derive;

pub mod commands;
pub mod game_events;
pub mod game;
pub mod math;
pub mod pops;
pub mod probability;
pub mod render;
pub mod storage;
pub mod ui;
pub mod world;
pub mod worldgen;
pub mod agent;

pub use commands::*;
pub use game_events::*;
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
pub use agent::*;

fn main() {
    let cb = ContextBuilder::new("iron-age", "ristew")
        .window_setup(WindowSetup::default().vsync(false).title("iron-age"))
        .window_mode(WindowMode::default().dimensions(1150.0, 750.0));
    let (mut ctx, evt_loop) = cb.build().unwrap();
    let game = game::MainState::new(&mut ctx);
    event::run(ctx, evt_loop, game);
}
