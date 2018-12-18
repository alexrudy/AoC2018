use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

use crate::geometry::{BoundingBox, Point};

#[derive(Debug, Clone, PartialEq, Eq, Fail)]
pub enum ParseTileError {
    #[fail(display = "No characters to parse")]
    NoCharacters,

    #[fail(display = "Unknown Tile: {}", _0)]
    UnknownTile(String),

    #[fail(display = "Too many characters to parse: {}", _0)]
    TooManyCharacters(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Empty,
    Wall,
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Tile::Empty => write!(f, "."),
            Tile::Wall => write!(f, "#"),
        }
    }
}

impl FromStr for Tile {
    type Err = ParseTileError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseTileError::NoCharacters);
        }
        if s.len() != 1 {
            return Err(ParseTileError::TooManyCharacters(s.to_string()));
        }

        match s {
            "." => Ok(Tile::Empty),
            "#" => Ok(Tile::Wall),
            _ => Err(ParseTileError::UnknownTile(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid(HashSet<Point>);

impl Default for Grid {
    fn default() -> Self {
        Self::new()
    }
}

impl Grid {
    pub fn new() -> Self {
        Grid(HashSet::new())
    }

    pub fn insert(&mut self, point: Point, tile: Tile) -> bool {
        match tile {
            Tile::Empty => self.0.insert(point),
            Tile::Wall => self.0.remove(&point),
        }
    }

    pub fn get(&self, point: Point) -> Tile {
        if self.0.contains(&point) {
            Tile::Empty
        } else {
            Tile::Wall
        }
    }

    pub fn bbox(&self) -> BoundingBox {
        let mut bbox = BoundingBox::empty();
        for point in &self.0 {
            bbox.include(*point);
        }
        bbox
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile() {
        assert_eq!(".".parse::<Tile>(), Ok(Tile::Empty));
        assert_eq!("#".parse::<Tile>(), Ok(Tile::Wall));
        assert_eq!("".parse::<Tile>(), Err(ParseTileError::NoCharacters));
        assert_eq!(
            "B".parse::<Tile>(),
            Err(ParseTileError::UnknownTile("B".to_string()))
        );
        assert_eq!(
            "Blah".parse::<Tile>(),
            Err(ParseTileError::TooManyCharacters("Blah".to_string()))
        );

        assert_eq!(&format!("{}", Tile::Empty), ".");
        assert_eq!(&format!("{}", Tile::Wall), "#");
    }

    #[test]
    fn grid() {
        let mut g = Grid::new();

        let position = Point::new(1, 1);
        assert!(g.insert(position, Tile::Empty));
        assert_eq!(g.get(position), Tile::Empty);
        assert!(g.insert(position, Tile::Wall));
        assert_eq!(g.get(position), Tile::Wall);
    }
}
