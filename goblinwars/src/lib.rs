#![feature(try_from)]

#[macro_use]
extern crate failure;

extern crate cursive;

#[macro_use]
extern crate lazy_static;
extern crate regex;

mod examples;
mod geometry;
pub mod map;
mod sprite;
pub mod views;

pub use self::examples::CombatExample;
