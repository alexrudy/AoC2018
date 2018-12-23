#![allow(dead_code)]

use std::cmp;
use std::fmt;
use std::ops::RangeInclusive;
use std::str::FromStr;

use failure::Fail;
use lazy_static::lazy_static;
use regex::Regex;

pub type Position = i32;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Left,
    Direction::Right,
    Direction::Down,
];

impl Direction {
    /// Enumertates all directions of movement in "reading order",
    /// i.e. such that the resulting points are in reading order
    /// from the current position.
    pub fn all() -> impl Iterator<Item = Self> {
        DIRECTIONS.iter().cloned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: Position,
    pub y: Position,
}

impl Point {
    pub fn new(x: Position, y: Position) -> Self {
        Self { x, y }
    }

    pub fn reading_order(self, other: Point) -> cmp::Ordering {
        self.y.cmp(&other.y).then(self.x.cmp(&other.x)).reverse()
    }

    fn up(self) -> Self {
        Self {
            x: self.x,
            y: self.y - 1,
        }
    }

    fn down(self) -> Self {
        Self {
            x: self.x,
            y: self.y + 1,
        }
    }

    fn left(self) -> Self {
        Self {
            x: self.x - 1,
            y: self.y,
        }
    }

    fn right(self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y,
        }
    }

    pub fn step(self, direction: Direction) -> Self {
        match direction {
            Direction::Left => self.left(),
            Direction::Right => self.right(),
            Direction::Up => self.up(),
            Direction::Down => self.down(),
        }
    }

    pub fn adjacent(self) -> impl Iterator<Item = Self> {
        Direction::all().map(move |d| self.step(d))
    }

    pub fn manhattan_distance(self, other: Point) -> Position {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

impl cmp::Ord for Point {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.y.cmp(&other.y).then(self.x.cmp(&other.x))
    }
}

impl cmp::PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

#[derive(Debug, Fail)]
pub enum ParsePointError {
    #[fail(display = "Invalid Point: {}", _0)]
    InvalidLiteral(String),

    #[fail(display = "Invalid Number Literal")]
    InvalidNumber,
}

impl From<::std::num::ParseIntError> for ParsePointError {
    fn from(_: ::std::num::ParseIntError) -> Self {
        ParsePointError::InvalidNumber
    }
}

impl FromStr for Point {
    type Err = ParsePointError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?P<x>[\d]+),\s*(?P<y>[\d]+)").unwrap();
        };

        let cap = match RE.captures(s) {
            None => return Err(ParsePointError::InvalidLiteral(s.to_string())),
            Some(c) => c,
        };

        Ok(Self::new(cap["x"].parse()?, cap["y"].parse()?))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundingBox {
    left: Position,
    right: Position,
    top: Position,
    bottom: Position,
}

impl BoundingBox {
    pub fn empty() -> Self {
        Self {
            left: Position::max_value(),
            right: Position::min_value(),
            top: Position::max_value(),
            bottom: Position::min_value(),
        }
    }

    pub fn zero() -> Self {
        Self {
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
        }
    }

    pub fn include(&mut self, point: Point) {
        if point.x < self.left {
            self.left = point.x;
        }
        if point.x > self.right {
            self.right = point.x;
        }
        if point.y < self.top {
            self.top = point.y;
        }
        if point.y > self.bottom {
            self.bottom = point.y;
        }
    }

    pub fn from_points(points: impl Iterator<Item = Point>) -> Self {
        let mut bbox = Self::empty();
        for point in points {
            bbox.include(point);
        }
        bbox
    }

    pub fn union(&self, other: &Self) -> Self {
        Self {
            left: if self.left > other.left {
                other.left
            } else {
                self.left
            },
            right: if self.right > other.right {
                self.right
            } else {
                other.right
            },
            top: if self.top > other.top {
                other.top
            } else {
                self.top
            },
            bottom: if self.bottom > other.bottom {
                self.bottom
            } else {
                other.bottom
            },
        }
    }

    pub fn margin(&self, size: Position) -> Self {
        Self {
            left: self.left - size,
            right: self.right + size,
            top: self.top - size,
            bottom: self.bottom + size,
        }
    }

    pub fn horizontal_margin(&self, size: Position) -> Self {
        Self {
            left: self.left - size,
            right: self.right + size,
            top: self.top,
            bottom: self.bottom,
        }
    }

    pub fn vertical_margin(&self, size: Position) -> Self {
        Self {
            left: self.left,
            right: self.right,
            top: self.top - size,
            bottom: self.bottom + size,
        }
    }

    pub fn vertical(&self) -> RangeInclusive<Position> {
        self.top..=self.bottom
    }

    pub fn horizontal(&self) -> RangeInclusive<Position> {
        self.left..=self.right
    }

    pub fn contains(&self, point: Point) -> bool {
        (point.x >= self.left)
            && (point.x <= self.right)
            && (point.y >= self.top)
            && (point.y <= self.bottom)
    }

    pub fn width(&self) -> Position {
        self.right.saturating_sub(self.left) + 1
    }

    pub fn height(&self) -> Position {
        self.bottom.saturating_sub(self.top) + 1
    }

    pub fn left(&self) -> Position {
        self.left
    }

    pub fn right(&self) -> Position {
        self.right
    }

    pub fn top(&self) -> Position {
        self.top
    }

    pub fn bottom(&self) -> Position {
        self.bottom
    }

    pub fn is_edge(&self, point: Point) -> bool {
        point.x == self.left
            || point.x == self.right
            || point.y == self.top
            || point.y == self.bottom
    }

    pub fn points(&self) -> BoundingBoxIterator {
        BoundingBoxIterator {
            bbox: self,
            px: 0,
            py: 0,
        }
    }
}

pub struct BoundingBoxIterator<'b> {
    bbox: &'b BoundingBox,
    px: i32,
    py: i32,
}

impl<'b> Iterator for BoundingBoxIterator<'b> {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.px == self.bbox.width() {
            self.px = 0;
            self.py += 1;
        }

        if self.py >= self.bbox.height() {
            return None;
        }

        let result = Some(Point {
            x: self.px + self.bbox.left(),
            y: self.py + self.bbox.top(),
        });
        self.px += 1;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point() {
        let point = Point::new(1, 1);

        assert_eq!(point.step(Direction::Up), Point::new(1, 0));
        assert_eq!(point.step(Direction::Down), Point::new(1, 2));
        assert_eq!(point.step(Direction::Left), Point::new(0, 1));
        assert_eq!(point.step(Direction::Right), Point::new(2, 1));

        assert_eq!(&point.to_string(), "1,1");
    }

    #[test]
    fn direction() {
        let origin = Point::new(0, 0);

        let mut steps = Vec::new();
        for direction in Direction::all() {
            steps.push(origin.step(direction));
        }

        assert_eq!(
            steps,
            vec![
                Point::new(0, -1),
                Point::new(-1, 0),
                Point::new(1, 0),
                Point::new(0, 1)
            ]
        );

        let mut others = steps.clone();
        others.reverse();
        steps.sort_by(|s, o| s.reading_order(*o));
        assert_eq!(steps, others);
    }

    #[test]
    fn bbox() {
        let mut bbox = BoundingBox::empty();

        let point = Point::new(1, 2);

        bbox.include(point);
        assert_eq!(bbox.left(), 1);
        assert_eq!(bbox.right(), 1);
        assert_eq!(bbox.width(), 1);
        assert_eq!(bbox.top(), 2);
        assert_eq!(bbox.bottom(), 2);
        assert_eq!(bbox.height(), 1);

        assert_eq!(bbox.horizontal(), 1..=1);
        assert_eq!(bbox.vertical(), 2..=2);

        bbox.include(Point::new(2, 2));

        assert_eq!(bbox.left(), 1);
        assert_eq!(bbox.right(), 2);
        assert_eq!(bbox.width(), 2);
        assert_eq!(bbox.top(), 2);
        assert_eq!(bbox.bottom(), 2);
        assert_eq!(bbox.height(), 1);

        assert_eq!(bbox.horizontal(), 1..=2);
        assert_eq!(bbox.vertical(), 2..=2);

        let other_bbox = BoundingBox {
            left: 3,
            right: 5,
            top: 0,
            bottom: 2,
        };

        assert!(!other_bbox.contains(point));

        let combined = bbox.union(&other_bbox);
        assert!(combined.contains(point));
        assert_eq!(combined.left(), 1);
        assert_eq!(combined.right(), 5);
        assert_eq!(combined.width(), 5);
        assert_eq!(combined.top(), 0);
        assert_eq!(combined.bottom(), 2);
        assert_eq!(combined.height(), 3);
    }
}
