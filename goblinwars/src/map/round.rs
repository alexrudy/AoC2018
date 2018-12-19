use std::collections::BinaryHeap;
use std::fmt;

use crate::geometry::{Direction, Point};
use crate::sprite::{Health, Species, SpriteStatus};

use super::pathfinding::Pathfinder;
use super::Map;

use failure::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoundOutcome {
    NoAction,
    CombatOnly,
    Casualty,
    Movement,
    MidRoundVictory(Species),
    Victory(Species),
}

impl RoundOutcome {
    fn combat(self) -> Self {
        match self {
            RoundOutcome::NoAction => RoundOutcome::CombatOnly,
            others => others,
        }
    }

    fn movement(self) -> Self {
        match self {
            RoundOutcome::NoAction => RoundOutcome::Movement,
            RoundOutcome::CombatOnly => RoundOutcome::Movement,
            others => others,
        }
    }

    fn casualty(self) -> Self {
        match self {
            RoundOutcome::CombatOnly => RoundOutcome::Casualty,
            RoundOutcome::NoAction => RoundOutcome::Casualty,
            others => others,
        }
    }

    fn is_finished(self) -> bool {
        match self {
            RoundOutcome::CombatOnly => false,
            RoundOutcome::Movement => false,
            RoundOutcome::Casualty => false,
            RoundOutcome::NoAction => false,
            _ => true,
        }
    }

    fn is_stable(self) -> bool {
        match self {
            RoundOutcome::CombatOnly => true,
            _ => false,
        }
    }
}

#[derive(Debug, Fail)]
pub enum RoundError {
    #[fail(display = "No moves remain on the map.")]
    NoMovesRemain,

    #[fail(display = "Game interrupted: {}", _0)]
    Interrupted(Box<Error>),
}

pub struct Round<'m> {
    map: &'m mut Map,
    queue: BinaryHeap<Point>,
}

impl<'m> Round<'m> {
    pub fn new(map: &'m mut Map) -> Self {
        let queue = map.sprites.positions().cloned().collect();
        Self { map, queue }
    }

    pub fn play(mut self) -> RoundOutcome {
        let mut outcome = RoundOutcome::NoAction;

        while (!self.queue.is_empty()) && (!outcome.is_finished()) {
            outcome = self.tick(outcome);
        }

        // Check if we hit a end-round victory
        if let Some(victor) = self.map.victorious() {
            if !outcome.is_finished() {
                return RoundOutcome::Victory(victor);
            }
        }

        outcome
    }

    fn direction(&self, location: Point, _outcome: RoundOutcome) -> Option<Direction> {
        if self.map.target(location).is_none() {
            self.map
                .pathfinder
                .find_path(self.map, location)
                .map(|path| path.direction())
        } else {
            None
        }
    }

    pub fn tick(&mut self, outcome: RoundOutcome) -> RoundOutcome {
        let mut outcome = outcome;
        if let Some(victor) = self.map.victorious() {
            return RoundOutcome::MidRoundVictory(victor);
        }

        if let Some(location) = self.queue.pop() {
            // First, the pathfinding phase
            let direction = self.direction(location, outcome);

            // Now movement
            let location = if let Some(d) = direction {
                self.map.sprites.step(location, d);
                outcome = outcome.movement();
                self.map.pathfinder.clear();
                location.step(d)
            } else {
                location
            };

            // Next, the attack phase
            if let Some(target) = self.map.target(location) {
                outcome = match self.map.sprites.attack(location, target) {
                    SpriteStatus::Alive(_) => outcome.combat(),
                    SpriteStatus::Dead => outcome.casualty(),
                };
            }
        }

        outcome
    }
}

#[derive(Debug)]
pub struct RunOutcome {
    pub victors: Species,
    pub rounds: u32,
    pub score: Health,
}

impl fmt::Display for RunOutcome {
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

impl Map {
    pub(crate) fn round(&mut self) -> Round {
        Round::new(self)
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
                        score: round * self.score(),
                    })
                }
                RoundOutcome::MidRoundVictory(s) => {
                    return Ok(RunOutcome {
                        rounds: round - 1,
                        victors: s,
                        score: (round - 1) * self.score(),
                    })
                }
                RoundOutcome::NoAction => return Err(RoundError::NoMovesRemain),
                _ => {}
            }
        }
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::MapBuilder;

    use crate::examples::map_ascii_trim;

    macro_rules! example_map {
        ($n:expr) => {
            include_str!(concat!("../../examples/", $n, ".txt"))
        };
    }

    fn trim(s: &str) -> String {
        map_ascii_trim(s)
    }

    #[test]
    fn movement() {
        let builder = MapBuilder::default();
        let mut example_map = builder.build(example_map!("movement/1")).unwrap();

        let maps = vec![
            example_map!("movement/1"),
            example_map!("movement/2"),
            example_map!("movement/3"),
            example_map!("movement/4"),
        ];

        assert_eq!(
            example_map
                .round()
                .direction(Point::new(1, 1), RoundOutcome::NoAction),
            Some(Direction::Right)
        );

        // assert_eq!(
        //     example_map.direction(Point::new(1, 1), RoundOutcome::CombatOnly),
        //     None
        // );

        assert_eq!(
            example_map
                .round()
                .direction(Point::new(1, 1), RoundOutcome::Casualty),
            Some(Direction::Right)
        );

        assert_eq!(trim(maps[0]), trim(&example_map.to_string()));

        {
            let mut em = example_map.clone();
            ;
            assert_eq!(
                em.round().tick(RoundOutcome::NoAction),
                RoundOutcome::Movement
            );
        }

        for (i, raw_map) in maps.iter().enumerate() {
            assert_eq!(
                trim(&example_map.to_string()),
                trim(raw_map),
                "Map doesn't line up at {}\ngot:\n{}\nexpected:\n{}",
                i,
                trim(&example_map.to_string()),
                trim(raw_map)
            );
            example_map.round().play();
        }

        assert_eq!(trim(maps[3]), trim(&example_map.to_string()));
    }

}
