use std::cmp::Ordering;

use failure::Fail;

use geometry::{Direction, Point};

use crate::layout::Track;

#[derive(Debug, Fail, PartialEq)]
pub enum CartError {
    #[fail(display = "Off the rails at {}", _0)]
    OffTheRails(Point),

    #[fail(display = "Collision at {}", _0)]
    Collision(Point),
}

impl CartError {
    pub(crate) fn from_advance(error: CartAdvanceError, position: Point) -> Self {
        match error {
            CartAdvanceError::OffTheRails => CartError::OffTheRails(position),
        }
    }
}

#[derive(Debug, Fail, PartialEq)]
pub(crate) enum CartAdvanceError {
    #[fail(display = "Off the rails")]
    OffTheRails,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Turn {
    Left,
    Right,
    Straight,
}

impl Turn {
    fn next(self) -> Self {
        match self {
            Turn::Left => Turn::Straight,
            Turn::Straight => Turn::Right,
            Turn::Right => Turn::Left,
        }
    }

    fn advance(self, direction: Direction) -> (Self, Direction) {
        let direction = match (self, direction) {
            (Turn::Left, Direction::Left) => Direction::Down,
            (Turn::Left, Direction::Down) => Direction::Right,
            (Turn::Left, Direction::Right) => Direction::Up,
            (Turn::Left, Direction::Up) => Direction::Left,
            (Turn::Straight, _) => direction,
            (Turn::Right, Direction::Left) => Direction::Up,
            (Turn::Right, Direction::Down) => Direction::Left,
            (Turn::Right, Direction::Right) => Direction::Down,
            (Turn::Right, Direction::Up) => Direction::Right,
        };
        (self.next(), direction)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct Cart {
    pub(crate) position: Point,
    pub(crate) direction: Direction,
    intersection: Turn,
}

const GRIDSIZE: i32 = 1000;

impl PartialOrd for Cart {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Cart {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.position.y * GRIDSIZE + self.position.x)
            .cmp(&(other.position.y * GRIDSIZE + other.position.x))
            .reverse()
    }
}

impl Cart {
    pub(crate) fn new(position: Point, direction: Direction) -> Self {
        Self {
            position,
            direction,
            intersection: Turn::Left,
        }
    }

    fn direction(&mut self, track: Track) -> Result<Direction, CartAdvanceError> {
        match (self.direction, track) {
            (_, Track::Empty) => Err(CartAdvanceError::OffTheRails),
            (Direction::Left, Track::Horizontal) => Ok(Direction::Left),
            (Direction::Right, Track::Horizontal) => Ok(Direction::Right),
            (_, Track::Horizontal) => Err(CartAdvanceError::OffTheRails),
            (Direction::Up, Track::Vertical) => Ok(Direction::Up),
            (Direction::Down, Track::Vertical) => Ok(Direction::Down),
            (_, Track::Vertical) => Err(CartAdvanceError::OffTheRails),
            (Direction::Up, Track::LeftCorner) => Ok(Direction::Left),
            (Direction::Right, Track::LeftCorner) => Ok(Direction::Down),
            (Direction::Down, Track::LeftCorner) => Ok(Direction::Right),
            (Direction::Left, Track::LeftCorner) => Ok(Direction::Up),
            (Direction::Up, Track::RightCorner) => Ok(Direction::Right),
            (Direction::Right, Track::RightCorner) => Ok(Direction::Up),
            (Direction::Down, Track::RightCorner) => Ok(Direction::Left),
            (Direction::Left, Track::RightCorner) => Ok(Direction::Down),
            (d, Track::Intersection) => {
                let (turn, d) = self.intersection.advance(d);
                self.intersection = turn;
                Ok(d)
            }
        }
    }

    pub(crate) fn advance(&mut self, track: Track) -> Result<(), CartAdvanceError> {
        self.direction = self.direction(track)?;
        self.position = self.position.step(self.direction);

        Ok(())
    }
}
