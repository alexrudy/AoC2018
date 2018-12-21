use failure::Error;

use crate::map::Map;
use crate::map::Pathfinders;

pub mod round;
use self::round::{Round, RoundError, RoundOutcome, RunOutcome};

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

    pub fn run<F>(&mut self, mut f: F) -> Result<RunOutcome, RoundError>
    where
        F: FnMut(&Self, u32) -> Result<(), Box<Error>>,
    {
        for round in 1.. {
            f(&self, round).map_err(RoundError::Interrupted)?;
            match self.round().play() {
                RoundOutcome::Victory(s) => {
                    return Ok(RunOutcome {
                        rounds: round,
                        victors: s,
                        score: round * self.map.score(),
                    })
                }
                RoundOutcome::MidRoundVictory(s) => {
                    return Ok(RunOutcome {
                        rounds: round - 1,
                        victors: s,
                        score: (round - 1) * self.map.score(),
                    })
                }
                RoundOutcome::NoAction => return Err(RoundError::NoMovesRemain),
                _ => {}
            }
        }
        unreachable!()
    }
}
