use failure::Error;
use std::fmt;

use crate::map::Map;
use crate::map::Pathfinders;
use crate::sprite::{Health, Species};

pub mod round;
use self::round::{Round, RoundError, RoundOutcome};

#[derive(Debug)]
pub enum GameOutcome {
    Complete(GameComplete),
    Stopped,
}

#[derive(Debug)]
pub struct GameComplete {
    pub victors: Species,
    pub rounds: u32,
    pub score: Health,
}

impl fmt::Display for GameComplete {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} win after {} rounds for a total score of {}",
            self.victors.plural(),
            self.rounds,
            self.score
        )
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    map: Map,
    pathfinders: Pathfinders,
}

impl Game {
    pub fn new(map: Map) -> Self {
        Self {
            map: map,
            pathfinders: Pathfinders::new(),
        }
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn round(&mut self) -> Round {
        Round::new(&mut self.map, &mut self.pathfinders)
    }

    pub fn run<F>(&mut self, mut f: F) -> Result<GameOutcome, RoundError>
    where
        F: FnMut(&Self, u32) -> Result<(), Box<Error>>,
    {
        for round in 1.. {
            f(&self, round).map_err(RoundError::Interrupted)?;
            match self.round().play() {
                RoundOutcome::Victory(s) => {
                    return Ok(GameOutcome::Complete(GameComplete {
                        rounds: round,
                        victors: s,
                        score: round * self.map.score(),
                    }))
                }
                RoundOutcome::MidRoundVictory(s) => {
                    return Ok(GameOutcome::Complete(GameComplete {
                        rounds: round - 1,
                        victors: s,
                        score: (round - 1) * self.map.score(),
                    }))
                }
                RoundOutcome::NoAction => return Err(RoundError::NoMovesRemain),
                _ => {}
            }
        }
        unreachable!()
    }
}
