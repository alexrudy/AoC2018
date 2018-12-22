use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::TryFrom;
use std::iter::FromIterator;

use geometry::{Direction, Point, Position};

use super::Map;
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

    pub fn step(&self, direction: Direction) -> Self {
        Self {
            destination: self.destination.step(direction),
            direction: self.direction,
            distance: self.distance + 1,
        }
    }

    pub fn extend(&self, extension: &Self) -> Self {
        Self {
            destination: extension.destination,
            direction: self.direction,
            distance: self.distance + extension.distance,
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
struct PathSegment {
    direction: Direction,
    path: Vec<Point>,
}

impl PathSegment {
    fn new(origin: Point, direction: Direction) -> Self {
        Self {
            direction,
            path: vec![origin],
        }
    }

    pub fn destination(&self) -> Point {
        *self.path.iter().last().unwrap()
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn distance(&self) -> Position {
        Position::try_from(self.path.len()).unwrap()
    }

    pub fn step(&self, direction: Direction) -> Self {
        let mut path = self.clone();
        path.path.push(self.destination().step(direction));
        path
    }

    pub fn extend(&self, extension: &Self) -> Self {
        let mut path = self.clone();
        path.path.extend(extension.path.iter().cloned());
        path
    }

    pub fn paths(point: Point) -> Vec<Self> {
        let mut paths = Vec::with_capacity(4);
        for direction in Direction::all() {
            paths.push(Self::new(point.step(direction), direction));
        }
        paths
    }
}

impl From<PathSegment> for SpritePath {
    fn from(ps: PathSegment) -> SpritePath {
        SpritePath::new(ps.destination(), ps.direction(), ps.distance())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pathfinders {
    pathfinders: HashMap<Species, Pathfinder>,
}

impl Default for Pathfinders {
    fn default() -> Self {
        Self::new()
    }
}

impl Pathfinders {
    pub fn new() -> Self {
        Self {
            pathfinders: HashMap::new(),
        }
    }

    pub fn get(&mut self, species: Species) -> &Pathfinder {
        self.pathfinders
            .entry(species)
            .or_insert_with(Pathfinder::new)
    }

    pub fn clear(&mut self) {
        for pf in self.pathfinders.values() {
            pf.clear()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pathfinder {
    path_cache: RefCell<HashMap<Point, Option<SpritePath>>>,
}

impl Default for Pathfinder {
    fn default() -> Self {
        Self::new()
    }
}

impl Pathfinder {
    pub fn new() -> Self {
        Self {
            path_cache: RefCell::from(HashMap::new()),
        }
    }

    fn calculate_partial_path(
        &self,
        map: &Map,
        candidate: PathSegment,
        visited: &mut HashSet<Point>,
    ) -> Vec<PathSegment> {
        let mut paths = Vec::new();
        for direction in Direction::all() {
            let next_point = candidate.destination().step(direction);
            if !visited.contains(&next_point) && !map.is_occupied(next_point) {
                paths.push(candidate.step(direction));
                visited.insert(next_point);
            }
        }

        paths
    }

    fn calculate_shortest_path(&self, map: &Map, origin: Point) -> Option<SpritePath> {
        let species = map.sprites.get(origin)?.species();

        if map.target(origin).is_some() {
            return None;
        }

        let targets = map.target_points(species);

        let mut visited = HashSet::new();

        // Candidate cached paths.
        let mut candidates = Vec::new();

        let mut paths = VecDeque::from_iter(
            PathSegment::paths(origin)
                .into_iter()
                .filter(|p| !map.is_occupied(p.destination())),
        );

        while !paths.is_empty() {
            let candidate = paths.pop_front().unwrap();

            // Novel destination case
            if targets.contains(&candidate.destination()) {
                candidates.push(candidate);
                continue;
            }

            paths.extend(
                self.calculate_partial_path(map, candidate, &mut visited)
                    .into_iter(),
            );
        }

        candidates
            .into_iter()
            .min_by_key(|c| c.distance())
            .map(|c| c.into())
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

    #[test]
    fn pathfinding_cache_line() {
        let builder = MapBuilder::default();
        let raw_map = example_map!("pathfinding_cache_line");
        let example_map = builder.build(raw_map).unwrap();

        assert_eq!(trim(raw_map), trim(&example_map.to_string()));
        let pathfinder = Pathfinder::new();
        let path = pathfinder.find_path(&example_map, Point::new(1, 1));
        assert_eq!(path, None);

        let path = pathfinder.find_path(&example_map, Point::new(1, 2));
        assert_eq!(path, None);

        let path = pathfinder.find_path(&example_map, Point::new(5, 3));
        assert_eq!(
            path,
            Some(SpritePath {
                destination: Point::new(1, 11),
                direction: Direction::Down,
                distance: 12,
            })
        );

        let path = pathfinder.find_path(&example_map, Point::new(4, 9));
        assert_eq!(
            path,
            Some(SpritePath {
                destination: Point::new(2, 12),
                direction: Direction::Down,
                distance: 7,
            })
        );
    }

}
