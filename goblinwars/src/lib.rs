#![feature(try_from)]

#[macro_use]
extern crate failure;

extern crate cursive;

#[macro_use]
extern crate lazy_static;
extern crate regex;

mod examples;
mod game;
pub mod geometry;
pub mod map;
pub mod sprite;
pub mod views;

pub use self::examples::CombatExample;
pub use self::game::{Game, GameOutcome};
