use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::str::FromStr;

use failure::{format_err, Error, Fail};

use geometry::{self, Point};

pub(crate) fn main() -> Result<(), Error> {
    use crate::input_to_string;

    let pattern = input_to_string(20)?;

    let map = parse_regex(&pattern)?;
    println!(
        "Part 1: {}",
        map.farthest_room()
            .ok_or_else(|| format_err!("No rooms found!"))?
    );

    println!(
        "Part 2: {}",
        map.rooms().filter(|r| r.distance >= 1000).count()
    );

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl From<geometry::Direction> for Direction {
    fn from(gd: geometry::Direction) -> Self {
        match gd {
            geometry::Direction::Up => Direction::North,
            geometry::Direction::Down => Direction::South,
            geometry::Direction::Left => Direction::West,
            geometry::Direction::Right => Direction::East,
        }
    }
}

impl From<Direction> for geometry::Direction {
    fn from(d: Direction) -> Self {
        match d {
            Direction::North => geometry::Direction::Up,
            Direction::South => geometry::Direction::Down,
            Direction::East => geometry::Direction::Right,
            Direction::West => geometry::Direction::Left,
        }
    }
}

#[derive(Debug, Fail)]
enum ParseDirectionError {
    #[fail(display = "Invalid direction {}", _0)]
    InvalidDirection(String),
}

impl FromStr for Direction {
    type Err = ParseDirectionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "N" => Ok(Direction::North),
            "S" => Ok(Direction::South),
            "E" => Ok(Direction::East),
            "W" => Ok(Direction::West),
            _ => Err(ParseDirectionError::InvalidDirection(s.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
struct Room {
    distance: usize,
}

impl Room {
    fn new(distance: usize) -> Self {
        Self { distance }
    }

    fn step(&self) -> Self {
        Self {
            distance: self.distance + 1,
        }
    }
}

#[derive(Debug, Clone)]
struct PathFragment {
    position: Point,
    room: Room,
}

impl PathFragment {
    fn new(position: Point, room: Room) -> Self {
        Self { position, room }
    }

    fn origin() -> Self {
        Self::new(Point::new(0, 0), Room::new(0))
    }

    fn step(&self, direction: Direction) -> Self {
        Self::new(self.position.step(direction.into()), self.room.step())
    }
}

#[derive(Debug, Clone)]
struct Map {
    rooms: BTreeMap<Point, Room>,
}

impl Map {
    fn new() -> Self {
        Self {
            rooms: BTreeMap::new(),
        }
    }

    fn insert(&mut self, path: &PathFragment) -> bool {
        match self.rooms.entry(path.position) {
            Entry::Vacant(e) => {
                e.insert(path.room.clone());
                true
            }
            Entry::Occupied(mut e) => {
                if e.get().distance > path.room.distance {
                    e.insert(path.room.clone());
                    true
                } else {
                    false
                }
            }
        }
    }

    fn rooms(&self) -> impl Iterator<Item = &Room> {
        self.rooms.values()
    }

    fn farthest_room(&self) -> Option<usize> {
        self.rooms().map(|r| r.distance).max()
    }
}

struct Parser {
    stack: Vec<PathFragment>,
}

impl Parser {
    fn new() -> Self {
        Self { stack: Vec::new() }
    }

    fn push(&mut self, fragment: PathFragment) {
        self.stack.push(fragment);
    }

    fn pop(&mut self) -> Option<PathFragment> {
        self.stack.pop()
    }

    fn peek(&mut self) -> Option<&PathFragment> {
        self.stack.last()
    }
}

#[derive(Debug, Fail)]
enum ParseError {
    #[fail(display = "Unbalanced paraentheses")]
    UnbalancedParens,

    #[fail(display = "Invalid Direction: {}", _0)]
    InvalidDirection(ParseDirectionError),
}

impl From<ParseDirectionError> for ParseError {
    fn from(error: ParseDirectionError) -> Self {
        ParseError::InvalidDirection(error)
    }
}

#[allow(deprecated)]
fn parse_regex(pattern: &str) -> Result<Map, ParseError> {
    let mut stack = Parser::new();
    let mut map = Map::new();

    let mut position = PathFragment::origin();

    for c in pattern
        .trim_left_matches('^')
        .trim_right_matches('$')
        .chars()
    {
        match c {
            '(' => {
                stack.push(position.clone());
            }
            ')' => {
                position = stack.pop().ok_or(ParseError::UnbalancedParens)?;
            }
            '|' => {
                position = stack.peek().ok_or(ParseError::UnbalancedParens)?.clone();
            }
            _ => {
                position = position.step(format!("{}", c).parse::<Direction>()?);
                map.insert(&position);
            }
        };
    }

    Ok(map)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn direction() {
        assert_eq!("N".parse::<Direction>().unwrap(), Direction::North);
    }

    #[test]
    fn parser() {
        assert_eq!(parse_regex("^WNE$").unwrap().farthest_room(), Some(3));

        assert_eq!(
            parse_regex("^ENWWW(NEEE|SSE(EE|N))$")
                .unwrap()
                .farthest_room(),
            Some(10)
        );

        assert_eq!(
            parse_regex("^ENNWSWW(NEWS|)SSSEEN(WNSE|)EE(SWEN|)NNN$")
                .unwrap()
                .farthest_room(),
            Some(18)
        );

        assert_eq!(
            parse_regex("^ESSWWN(E|NNENN(EESS(WNSE|)SSS|WWWSSSSE(SW|NNNE)))$")
                .unwrap()
                .farthest_room(),
            Some(23)
        );

        assert_eq!(
            parse_regex("^WSSEESWWWNW(S|NENNEEEENN(ESSSSW(NWSW|SSEN)|WSWWN(E|WWS(E|SS))))$")
                .unwrap()
                .farthest_room(),
            Some(31)
        )
    }

}
