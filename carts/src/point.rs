use std::fmt;
use std::str::FromStr;

use failure::Fail;

use geometry;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Direction(geometry::Direction);

impl From<geometry::Direction> for Direction {
    fn from(d: geometry::Direction) -> Direction {
        Direction(d)
    }
}

impl From<Direction> for geometry::Direction {
    fn from(d: Direction) -> Self {
        d.0
    }
}

#[derive(Debug, Fail)]
pub enum DirectionParseError {
    #[fail(display = "Wrong size direction sprite: {}", _0)]
    WrongSize(String),

    #[fail(display = "Unknown direction sprite: {}", _0)]
    UnkownCharacter(String),
}

impl FromStr for Direction {
    type Err = DirectionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            return Err(DirectionParseError::WrongSize(s.to_owned()));
        }

        match s.chars().nth(0).unwrap() {
            '^' => Ok(geometry::Direction::Up.into()),
            '>' => Ok(geometry::Direction::Right.into()),
            '<' => Ok(geometry::Direction::Left.into()),
            'v' => Ok(geometry::Direction::Down.into()),
            _ => Err(DirectionParseError::UnkownCharacter(s.to_owned())),
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c = match self.0 {
            geometry::Direction::Up => '^',
            geometry::Direction::Right => '>',
            geometry::Direction::Left => '<',
            geometry::Direction::Down => 'v',
        };
        write!(f, "{}", c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use geometry::Point;

    #[test]
    fn direction() {
        //Parse successfully
        assert_eq!(
            "^".parse::<Direction>().unwrap(),
            geometry::Direction::Up.into()
        );
        assert_eq!(
            ">".parse::<Direction>().unwrap(),
            geometry::Direction::Right.into()
        );
        assert_eq!(
            "<".parse::<Direction>().unwrap(),
            geometry::Direction::Left.into()
        );
        assert_eq!(
            "v".parse::<Direction>().unwrap(),
            geometry::Direction::Down.into()
        );

        //Parse failures
        assert_eq!(
            &"".parse::<Direction>().unwrap_err().to_string(),
            "Wrong size direction sprite: "
        );
        assert_eq!(
            &"abc".parse::<Direction>().unwrap_err().to_string(),
            "Wrong size direction sprite: abc"
        );
        assert_eq!(
            &"a".parse::<Direction>().unwrap_err().to_string(),
            "Unknown direction sprite: a"
        );

        //Display
        assert_eq!(Direction::from(geometry::Direction::Up).to_string(), "^");
        assert_eq!(Direction::from(geometry::Direction::Down).to_string(), "v");
        assert_eq!(Direction::from(geometry::Direction::Left).to_string(), "<");
        assert_eq!(Direction::from(geometry::Direction::Right).to_string(), ">");
    }

    #[test]
    fn point() {
        let point = Point::new(1, 1);

        assert_eq!(point.step(geometry::Direction::Up), Point::new(1, 0));
        assert_eq!(point.step(geometry::Direction::Down), Point::new(1, 2));
        assert_eq!(point.step(geometry::Direction::Left), Point::new(0, 1));
        assert_eq!(point.step(geometry::Direction::Right), Point::new(2, 1));

        assert_eq!(&point.to_string(), "1,1");
    }
}
