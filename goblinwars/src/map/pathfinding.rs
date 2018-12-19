use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};
use std::iter::FromIterator;

use super::Map;
use crate::geometry::{Direction, Point, Position};
use crate::sprite::Species;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpritePath {
    destination: Point,
    direction: Direction,
    distance: Position,
}

impl SpritePath {
    pub fn new(destination: Point, direction: Direction, distance: Position) -> Self {
        Self {
            destination,
            direction,
            distance,
        }
    }

    pub fn paths(point: Point) -> Vec<SpritePath> {
        let mut paths = Vec::with_capacity(4);
        for direction in Direction::all() {
            paths.push(SpritePath {
                destination: point.step(direction),
                direction: direction,
                distance: 1,
            })
        }
        paths
    }

    pub fn extend(&self, direction: Direction) -> Self {
        Self {
            destination: self.destination.step(direction),
            direction: self.direction,
            distance: self.distance + 1,
        }
    }

    pub fn destination(&self) -> Point {
        self.destination
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pathfinder {
    path_cache: RefCell<HashMap<Point, Option<SpritePath>>>,
}

impl Pathfinder {
    pub fn new() -> Self {
        Self {
            path_cache: RefCell::from(HashMap::new()),
        }
    }

    fn calculate_shortest_path(&self, map: &Map, origin: Point) -> Option<SpritePath> {
        let species = map.sprites.get(origin)?.species();

        if map.target(origin).is_some() {
            return None;
        }

        let targets = map.target_points(species);

        let mut visited = HashSet::new();

        let mut paths = VecDeque::from_iter(
            SpritePath::paths(origin)
                .into_iter()
                .filter(|p| map.element(p.destination()).is_empty()),
        );

        while !paths.is_empty() {
            let candidate = paths.pop_front().unwrap();

            if targets.contains(&candidate.destination()) {
                return Some(candidate);
            }

            visited.insert(candidate.destination());
            for direction in Direction::all() {
                let next_point = candidate.destination().step(direction);
                if !visited.contains(&next_point) && map.element(next_point).is_empty() {
                    paths.push_back(candidate.extend(direction))
                }
            }
        }

        None
    }

    /// Find a path between the origin point given and an enemy.
    pub fn find_path(&self, map: &Map, origin: Point) -> Option<SpritePath> {
        {
            match self.path_cache.borrow_mut().entry(origin) {
                Entry::Occupied(e) => {
                    return e.get().clone();
                }
                Entry::Vacant(e) => {
                    e.insert(self.calculate_shortest_path(map, origin));
                }
            }
        }

        self.find_path(map, origin)
    }

    pub(crate) fn clear(&self) {
        self.path_cache.borrow_mut().clear();
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
    fn pathfinding_simple() {
        let builder = MapBuilder::default();
        let raw_map = example_map!("pathfinding_simple");
        let example_map = builder.build(raw_map).unwrap();

        assert_eq!(trim(raw_map), trim(&example_map.to_string()));
        assert_eq!(example_map.sprites.len(), 4);

        let path = Pathfinder::new().find_path(&example_map, Point::new(1, 1));
        assert_eq!(
            path,
            Some(SpritePath {
                destination: Point::new(3, 1),
                direction: Direction::Right,
                distance: 2,
            })
        );
    }

    #[test]
    fn pathfinding_combat() {
        let builder = MapBuilder::default();
        let raw_map = example_map!("pathfinding_combat");
        let example_map = builder.build(raw_map).unwrap();

        assert_eq!(trim(raw_map), trim(&example_map.to_string()));
        assert_eq!(example_map.sprites.len(), 4);

        let path = Pathfinder::new().find_path(&example_map, Point::new(1, 1));
        assert_eq!(path, None);
    }

    #[test]
    fn pathfinding_multi() {
        let builder = MapBuilder::default();
        let raw_map = example_map!("pathfinding_multi");
        let example_map = builder.build(raw_map).unwrap();

        assert_eq!(trim(raw_map), trim(&example_map.to_string()));
        assert_eq!(example_map.sprites.len(), 2);

        let path = Pathfinder::new().find_path(&example_map, Point::new(2, 1));
        assert_eq!(
            path,
            Some(SpritePath {
                destination: Point::new(4, 2),
                direction: Direction::Right,
                distance: 3,
            })
        );
    }
}
