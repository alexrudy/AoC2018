#![feature(try_from)]

mod examples;
mod game;
pub mod map;
pub mod sprite;
pub mod views;

pub use self::examples::CombatExample;
pub use self::game::{Game, GameOutcome};
