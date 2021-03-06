use std::cell::RefCell;
use std::cmp;
use std::collections::{BTreeMap, BinaryHeap};
use std::num::ParseIntError;
use std::str::FromStr;

use failure::{format_err, Error, Fail};

use geometry::{BoundingBox, Direction, ParsePointError, Point, Position};

pub(crate) fn main() -> Result<(), Error> {
    use crate::input_to_string;

    let cave: Cave = input_to_string(22)?.parse()?;
    println!(
        "Part 1: {}",
        cave.risk_level(BoundingBox::from_corners(Point::new(0, 0), cave.target))
    );

    let minutes_to_rescue = shortest_path(&cave, Point::new(0, 0))
        .ok_or_else(|| format_err!("No path found to target!"))?;
    println!("Part 2: {}", minutes_to_rescue);
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Typ {
    Rocky,
    Wet,
    Narrow,
}

impl Typ {
    fn risk(self) -> usize {
        match self {
            Typ::Rocky => 0,
            Typ::Wet => 1,
            Typ::Narrow => 2,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Region {
    geologic_index: Position,
    erosion_index: Position,
}

impl Region {
    pub(crate) fn new(depth: Position, geologic: Position) -> Self {
        Self {
            geologic_index: geologic,
            erosion_index: (geologic + depth) % 20_183,
        }
    }

    pub(crate) fn typ(self) -> Typ {
        match self.erosion_index.abs() % 3 {
            0 => Typ::Rocky,
            1 => Typ::Wet,
            2 => Typ::Narrow,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
struct Cave {
    target: Point,
    depth: Position,

    // Used as a cache.
    cave_map: RefCell<BTreeMap<Point, Region>>,
}

impl Cave {
    pub(crate) fn new(target: Point, depth: Position) -> Self {
        Self {
            target,
            depth,
            cave_map: RefCell::from(BTreeMap::new()),
        }
    }

    fn geologic_index(&self, point: Point) -> Position {
        if point == Point::new(0, 0) {
            return 0;
        }
        if point == self.target {
            return 0;
        }
        if point.y == 0 {
            return point.x * 16_807;
        }

        if point.x == 0 {
            return point.y * 48_271;
        }

        {
            let left = self.region(point.step(Direction::Left));
            let above = self.region(point.step(Direction::Up));
            left.erosion_index * above.erosion_index
        }
    }

    pub(crate) fn region(&self, point: Point) -> Region {
        {
            if let Some(region) = self.cave_map.borrow().get(&point) {
                return *region;
            }
        }

        let region = Region::new(self.depth, self.geologic_index(point));
        {
            self.cave_map.borrow_mut().entry(point).or_insert(region);
        }
        region
    }

    pub(crate) fn risk_level(&self, bbox: BoundingBox) -> usize {
        bbox.points().map(|p| self.region(p).typ().risk()).sum()
    }
}

#[derive(Debug, Fail)]
enum CaveParseError {
    #[fail(display = "Pattern Failure: {}", _0)]
    Pattern(String),

    #[fail(display = "Parsing Integer")]
    ParseInt,

    #[fail(display = "Parsing Point: {}", _0)]
    PointError(ParsePointError),
}

impl From<ParseIntError> for CaveParseError {
    fn from(_e: ParseIntError) -> Self {
        CaveParseError::ParseInt
    }
}

impl From<ParsePointError> for CaveParseError {
    fn from(e: ParsePointError) -> Self {
        CaveParseError::PointError(e)
    }
}

impl FromStr for Cave {
    type Err = CaveParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let depth_line = lines
            .next()
            .ok_or_else(|| CaveParseError::Pattern(s.to_string()))?;
        let depth: Position = depth_line
            .split(':')
            .nth(1)
            .ok_or_else(|| CaveParseError::Pattern(s.to_string()))?
            .trim()
            .parse()?;
        let target_line = lines
            .next()
            .ok_or_else(|| CaveParseError::Pattern(s.to_string()))?;
        let target: Point = target_line
            .split(':')
            .nth(1)
            .ok_or_else(|| CaveParseError::Pattern(s.to_string()))?
            .trim()
            .parse()?;
        Ok(Self::new(target, depth))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Equipment {
    Nothing,
    ClimbingGear,
    Torch,
}

impl Equipment {
    fn passable(self, terrain: Typ) -> bool {
        match (self, terrain) {
            (Equipment::Nothing, Typ::Rocky) => false,
            (Equipment::Torch, Typ::Wet) => false,
            (Equipment::ClimbingGear, Typ::Narrow) => false,
            (_, _) => true,
        }
    }

    fn change_gear(self, terrain: Typ) -> Self {
        match (self, terrain) {
            (Equipment::ClimbingGear, Typ::Rocky) => Equipment::Torch,
            (Equipment::Torch, Typ::Rocky) => Equipment::ClimbingGear,
            (Equipment::Nothing, Typ::Wet) => Equipment::ClimbingGear,
            (Equipment::ClimbingGear, Typ::Wet) => Equipment::Nothing,
            (Equipment::Torch, Typ::Narrow) => Equipment::Nothing,
            (Equipment::Nothing, Typ::Narrow) => Equipment::Torch,
            (_, _) => panic!(
                "Illegal equipment / terrain combination: {:?}/{:?}",
                self, terrain
            ),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct PathFragment {
    position: Point,
    distance: usize,
    heuristic: usize,
    equipment: Equipment,
}

impl PathFragment {
    fn new(position: Point, distance: usize, heuristic: usize, equipment: Equipment) -> Self {
        Self {
            position,
            distance,
            heuristic,
            equipment,
        }
    }
}

impl cmp::Ord for PathFragment {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.heuristic
            .cmp(&other.heuristic)
            .reverse()
            .then(self.position.cmp(&other.position))
    }
}

impl cmp::PartialOrd for PathFragment {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
struct PathQueue {
    queue: BinaryHeap<PathFragment>,
    seen: BTreeMap<(Point, Equipment), usize>,
}

impl PathQueue {
    fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
            seen: BTreeMap::new(),
        }
    }

    fn push(&mut self, path: PathFragment) {
        match self.seen.get_mut(&(path.position, path.equipment)) {
            Some(d) => {
                if *d > path.distance {
                    *d = path.distance;
                    self.queue.push(path);
                }
            }
            None => {
                self.seen
                    .insert((path.position, path.equipment), path.distance);
                self.queue.push(path);
            }
        }
    }

    fn pop(&mut self) -> Option<PathFragment> {
        self.queue.pop()
    }
}

fn shortest_path(cave: &Cave, origin: Point) -> Option<usize> {
    let path = PathFragment::new(
        origin,
        0,
        origin.manhattan_distance(cave.target) as usize,
        Equipment::Torch,
    );
    let mut queue = PathQueue::new();

    queue.push(path);

    while let Some(path) = queue.pop() {
        if path.position == cave.target && path.equipment == Equipment::Torch {
            return Some(path.distance);
        }

        let current_typ = cave.region(path.position).typ();

        let change_equipment_path = PathFragment::new(
            path.position,
            path.distance + 7,
            path.heuristic + 7,
            path.equipment.change_gear(current_typ),
        );
        queue.push(change_equipment_path);

        for adjacent in path.position.adjacent() {
            if adjacent.x < 0 || adjacent.y < 0 {
                continue;
            }

            let typ = cave.region(adjacent).typ();

            if path.equipment.passable(typ) {
                let adjacent_path = PathFragment::new(
                    adjacent,
                    path.distance + 1,
                    path.distance + 1 + adjacent.manhattan_distance(cave.target) as usize,
                    path.equipment,
                );
                queue.push(adjacent_path);
            }
        }
    }

    // No appropriate path found.
    None
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example_part1() {
        let target = Point::new(10, 10);
        let cave = Cave::new(Point::new(10, 10), 510);
        assert_eq!(cave.region(Point::new(0, 0)).typ(), Typ::Rocky);
        assert_eq!(cave.region(Point::new(1, 0)).typ(), Typ::Wet);
        assert_eq!(cave.region(Point::new(0, 1)).typ(), Typ::Rocky);
        assert_eq!(cave.region(Point::new(1, 1)).typ(), Typ::Narrow);
        assert_eq!(cave.region(Point::new(10, 10)).typ(), Typ::Rocky);

        assert_eq!(
            cave.risk_level(BoundingBox::from_corners(Point::new(0, 0), target)),
            114
        );
    }

    #[test]
    fn example_part2() {
        let cave = Cave::new(Point::new(10, 10), 510);

        assert_eq!(shortest_path(&cave, Point::new(0, 0)), Some(45));
    }

}
