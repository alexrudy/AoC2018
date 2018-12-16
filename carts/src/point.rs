use std::error::Error;
use std::fmt;
use std::str::FromStr;

pub(crate) type Position = i32;

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<Error>::from(format!($($tt)*))) }
}

type ParseResult<T> = Result<T, Box<Error>>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl FromStr for Direction {
    type Err = Box<Error>;

    fn from_str(s: &str) -> ParseResult<Self> {
        if s.len() != 1 {
            return err!("Wrong size direction string");
        }

        match s.chars().nth(0).unwrap() {
            '^' => Ok(Direction::Up),
            '>' => Ok(Direction::Right),
            '<' => Ok(Direction::Left),
            'v' => Ok(Direction::Down),
            c => err!("Unknown direction: {}", c),
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c = match self {
            Direction::Up => '^',
            Direction::Right => '>',
            Direction::Left => '<',
            Direction::Down => 'v',
        };
        write!(f, "{}", c)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Point {
    pub x: Position,
    pub y: Position,
}

impl Point {
    pub fn new(x: Position, y: Position) -> Self {
        Self { x, y }
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

    pub(crate) fn advance(self, direction: Direction) -> Self {
        match direction {
            Direction::Left => self.left(),
            Direction::Right => self.right(),
            Direction::Up => self.up(),
            Direction::Down => self.down(),
        }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BBox {
    left: Position,
    right: Position,
    top: Position,
    bottom: Position,
}

impl BBox {
    pub fn empty() -> Self {
        Self {
            left: Position::max_value(),
            right: Position::min_value(),
            top: Position::max_value(),
            bottom: Position::min_value(),
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

    pub fn contains(&self, point: Point) -> bool {
        (point.x >= self.left)
            && (point.x <= self.right)
            && (point.y >= self.top)
            && (point.y <= self.bottom)
    }

    pub fn width(&self) -> Position {
        self.right - self.left + 1
    }

    pub fn height(&self) -> Position {
        self.bottom - self.top + 1
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction() {
        //Parse successfully
        assert_eq!("^".parse::<Direction>().unwrap(), Direction::Up);
        assert_eq!(">".parse::<Direction>().unwrap(), Direction::Right);
        assert_eq!("<".parse::<Direction>().unwrap(), Direction::Left);
        assert_eq!("v".parse::<Direction>().unwrap(), Direction::Down);

        //Parse failures
        assert_eq!(
            &"".parse::<Direction>().unwrap_err().to_string(),
            "Wrong size direction string"
        );
        assert_eq!(
            &"abc".parse::<Direction>().unwrap_err().to_string(),
            "Wrong size direction string"
        );
        assert_eq!(
            &"a".parse::<Direction>().unwrap_err().to_string(),
            "Unknown direction: a"
        );

        //Display
        assert_eq!(Direction::Up.to_string(), "^");
        assert_eq!(Direction::Down.to_string(), "v");
        assert_eq!(Direction::Left.to_string(), "<");
        assert_eq!(Direction::Right.to_string(), ">");
    }

    #[test]
    fn point() {
        let point = Point::new(1, 1);

        assert_eq!(point.advance(Direction::Up), Point::new(1, 0));
        assert_eq!(point.advance(Direction::Down), Point::new(1, 2));
        assert_eq!(point.advance(Direction::Left), Point::new(0, 1));
        assert_eq!(point.advance(Direction::Right), Point::new(2, 1));

        assert_eq!(&point.to_string(), "1,1");
    }
}
