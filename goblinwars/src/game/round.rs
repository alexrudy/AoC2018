use std::cmp;
use std::collections::BinaryHeap;

use failure::{Error, Fail};

use geometry::{Direction, Point};

use crate::map::Map;
use crate::map::Pathfinders;
use crate::sprite::{Species, SpriteStatus};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoundOutcome {
    NoAction,
    CombatOnly,
    Casualty(Species),
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

    fn casualty(self, species: Species) -> Self {
        match self {
            RoundOutcome::CombatOnly => RoundOutcome::Casualty(species),
            RoundOutcome::NoAction => RoundOutcome::Casualty(species),
            RoundOutcome::Movement => RoundOutcome::Casualty(species),
            others => others,
        }
    }

    fn is_finished(self) -> bool {
        match self {
            RoundOutcome::CombatOnly => false,
            RoundOutcome::Movement => false,
            RoundOutcome::Casualty(_) => false,
            RoundOutcome::NoAction => false,
            _ => true,
        }
    }
}

#[derive(Debug, Fail)]
pub enum RoundError {
    #[fail(display = "No moves remain on the map.")]
    NoMovesRemain,

    #[fail(display = "Game interrupted: {}", _0)]
    Interrupted(Error),
}

#[derive(Debug, PartialEq, Eq)]
struct QPoint(Point);

impl cmp::Ord for QPoint {
    fn cmp(&self, other: &QPoint) -> cmp::Ordering {
        self.0.cmp(&other.0).reverse()
    }
}

impl cmp::PartialOrd for QPoint {
    fn partial_cmp(&self, other: &QPoint) -> Option<cmp::Ordering> {
        Some(self.0.cmp(&other.0).reverse())
    }
}

impl From<Point> for QPoint {
    fn from(p: Point) -> QPoint {
        QPoint(p)
    }
}

impl From<QPoint> for Point {
    fn from(q: QPoint) -> Point {
        q.0
    }
}

pub struct Round<'m> {
    map: &'m mut Map,
    pathfinder: &'m mut Pathfinders,
    queue: BinaryHeap<QPoint>,
}

impl<'m> Round<'m> {
    pub fn new(map: &'m mut Map, pathfinder: &'m mut Pathfinders) -> Self {
        let queue = map.sprites.positions().cloned().map(|p| p.into()).collect();
        Self {
            map,
            queue,
            pathfinder,
        }
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

    fn direction(&mut self, location: Point, _outcome: RoundOutcome) -> Option<Direction> {
        if self.map.target(location).is_none() {
            let species = self.map.sprites.get(location)?.species();
            self.pathfinder
                .get(species)
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

        if let Some(location) = self.queue.pop().map(|q| q.into()) {
            // First, the pathfinding phase
            let direction = self.direction(location, outcome);

            // Now movement
            let location = if let Some(d) = direction {
                self.map.sprites.step(location, d);
                outcome = outcome.movement();
                self.pathfinder.clear();
                location.step(d)
            } else {
                location
            };

            // Next, the attack phase
            if let Some(target) = self.map.target(location) {
                outcome = match self.map.sprites.attack(location, target) {
                    SpriteStatus::Alive(_, _) => outcome.combat(),
                    SpriteStatus::Dead(species) => {
                        self.pathfinder.clear();
                        outcome.casualty(species)
                    }
                };
            }
        }

        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::game::Game;
    use crate::map::MapBuilder;

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
        let example_map = builder.build(example_map!("movement/1")).unwrap();
        let mut game = Game::new(example_map);

        let maps = vec![
            example_map!("movement/1"),
            example_map!("movement/2"),
            example_map!("movement/3"),
            example_map!("movement/4"),
        ];

        assert_eq!(
            game.round()
                .direction(Point::new(1, 1), RoundOutcome::NoAction),
            Some(Direction::Right)
        );

        assert_eq!(
            game.round()
                .direction(Point::new(1, 1), RoundOutcome::Casualty(Species::Elf)),
            Some(Direction::Right)
        );

        assert_eq!(trim(maps[0]), trim(&game.map().to_string()));

        {
            let mut g2 = game.clone();
            ;
            assert_eq!(
                g2.round().tick(RoundOutcome::NoAction),
                RoundOutcome::Movement
            );
        }

        for (i, raw_map) in maps.iter().enumerate() {
            assert_eq!(
                trim(&game.map().to_string()),
                trim(raw_map),
                "Map doesn't line up at {}\ngot:\n{}\nexpected:\n{}",
                i,
                trim(&game.map().to_string()),
                trim(raw_map)
            );
            game.round().play();
        }

        assert_eq!(trim(maps[3]), trim(&game.map().to_string()));
    }

}
