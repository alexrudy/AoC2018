use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::hash::Hash;
use std::mem;
use std::str::FromStr;

use failure::{Error, Fail};

use geometry::{BoundingBox, Point};

pub(crate) fn main() -> Result<(), Error> {
    use crate::input_to_string;

    let la: LumberArea = input_to_string(18)?.parse()?;

    println!(
        "Part 1: {}",
        la.clone().evolve().nth(10).unwrap().resource_value()
    );

    println!("Part 2: {}", part2(la));

    Ok(())
}

fn part2(lumber: LumberArea) -> usize {
    let target = 1_000_000_000;
    let pattern = repeated_pattern(lumber.clone().evolve().map(|l| l.to_string()));
    let offset = (target - pattern.start) % pattern.length;
    lumber
        .evolve()
        .skip(pattern.start)
        .nth(offset)
        .unwrap()
        .resource_value()
}

#[derive(Debug)]
struct RepeatedPatternResult<T> {
    first: T,
    length: usize,
    start: usize,
}

fn repeated_pattern<I, T>(iter: I) -> RepeatedPatternResult<T>
where
    I: Iterator<Item = T>,
    T: Hash + Eq + Ord + Clone,
{
    let mut seen = HashMap::new();

    for (i, item) in iter.enumerate() {
        if let Some(s) = seen.insert(item.clone(), i) {
            return RepeatedPatternResult {
                first: item,
                length: i - s,
                start: s,
            };
        }
    }
    unreachable!();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
enum Acre {
    OpenGround,
    Trees,
    Lumberyard,
}

impl fmt::Display for Acre {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Acre::OpenGround => write!(f, "."),
            Acre::Lumberyard => write!(f, "#"),
            Acre::Trees => write!(f, "|"),
        }
    }
}

#[derive(Debug, Fail)]
enum ParseAcreError {
    #[fail(display = "Unknown Acre: {}", _0)]
    Unknown(String),
}

impl FromStr for Acre {
    type Err = ParseAcreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "." => Ok(Acre::OpenGround),
            "#" => Ok(Acre::Lumberyard),
            "|" => Ok(Acre::Trees),
            _ => Err(ParseAcreError::Unknown(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct LumberArea {
    zone: BTreeMap<Point, Acre>,
    bbox: BoundingBox,
}

impl LumberArea {
    fn empty() -> Self {
        Self::with_bbox(BoundingBox::zero())
    }

    fn with_bbox(bbox: BoundingBox) -> Self {
        Self {
            zone: BTreeMap::new(),
            bbox,
        }
    }

    fn insert(&mut self, point: Point, acre: Acre) -> Option<Acre> {
        self.bbox.include(point);
        match acre {
            Acre::OpenGround => self.zone.remove(&point),
            a => self.zone.insert(point, a),
        }
    }

    fn get(&self, point: Point) -> Acre {
        self.zone.get(&point).map_or(Acre::OpenGround, |a| *a)
    }

    fn resource_value(&self) -> usize {
        let mut lumber = 0;
        let mut trees = 0;
        for acre in self.zone.values() {
            match acre {
                Acre::Lumberyard => {
                    lumber += 1;
                }
                Acre::Trees => {
                    trees += 1;
                }
                Acre::OpenGround => {}
            };
        }
        lumber * trees
    }

    fn bbox(&self) -> BoundingBox {
        self.bbox
    }

    fn evolve(self) -> LumberIterator {
        LumberIterator { area: self }
    }

    fn change(&self, point: Point) -> Acre {
        let a = self.get(point);
        match a {
            Acre::OpenGround => {
                if point
                    .adjacent_diagonal()
                    .filter(|p| self.get(*p) == Acre::Trees)
                    .count()
                    >= 3
                {
                    Acre::Trees
                } else {
                    Acre::OpenGround
                }
            }
            Acre::Trees => {
                if point
                    .adjacent_diagonal()
                    .filter(|p| self.get(*p) == Acre::Lumberyard)
                    .count()
                    >= 3
                {
                    Acre::Lumberyard
                } else {
                    Acre::Trees
                }
            }
            Acre::Lumberyard => {
                let neighbors: HashSet<Acre> =
                    point.adjacent_diagonal().map(|p| self.get(p)).collect();
                if neighbors.contains(&Acre::Lumberyard) && neighbors.contains(&Acre::Trees) {
                    Acre::Lumberyard
                } else {
                    Acre::OpenGround
                }
            }
        }
    }
}

impl FromStr for LumberArea {
    type Err = ParseAcreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut zone = LumberArea::empty();

        for (y, line) in s.lines().enumerate().take(50) {
            for (x, c) in line.chars().enumerate().take(50) {
                zone.insert(
                    Point::new(x as i32, y as i32),
                    c.to_string().parse::<Acre>()?,
                );
            }
        }

        Ok(zone)
    }
}

impl fmt::Display for LumberArea {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bbox = self.bbox();
        for y in bbox.vertical() {
            for x in bbox.horizontal() {
                write!(f, "{}", self.get(Point::new(x, y)))?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

struct LumberIterator {
    area: LumberArea,
}

impl Iterator for LumberIterator {
    type Item = LumberArea;

    fn next(&mut self) -> Option<LumberArea> {
        let bbox = self.area.bbox();
        let mut area = LumberArea::with_bbox(bbox);

        for y in bbox.vertical() {
            for x in bbox.horizontal() {
                let point = Point::new(x, y);
                area.insert(point, self.area.change(point));
            }
        }

        Some(mem::replace(&mut self.area, area))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn acre() {
        assert_eq!(".".parse::<Acre>().unwrap(), Acre::OpenGround);
        assert_eq!("#".parse::<Acre>().unwrap(), Acre::Lumberyard);
        assert_eq!("|".parse::<Acre>().unwrap(), Acre::Trees);
    }

    #[test]
    fn lumber_area() {
        let la: LumberArea = ".#\n|.".parse().unwrap();

        let mut lb = LumberArea::empty();
        lb.insert(Point::new(0, 0), Acre::OpenGround);
        lb.insert(Point::new(0, 1), Acre::Trees);
        lb.insert(Point::new(1, 0), Acre::Lumberyard);
        lb.insert(Point::new(1, 1), Acre::OpenGround);

        assert_eq!(la, lb);
    }

    #[test]
    fn example_part1() {
        let la: LumberArea = ".#.#...|#.
.....#|##|
.|..|...#.
..|#.....#
#.#|||#|#|
...#.||...
.|....|...
||...#|.#|
|.||||..|.
...#.|..|."
            .parse()
            .unwrap();

        eprintln!("{}", la);

        let lb = la.evolve().nth(10).unwrap();

        let lc: LumberArea = ".||##.....
||###.....
||##......
|##.....##
|##.....##
|##....##|
||##.####|
||#####|||
||||#|||||
||||||||||"
            .parse()
            .unwrap();
        eprintln!("{}", lb);
        eprintln!("{}", lc);
        assert_eq!(lb, lc);
        assert_eq!(lb.resource_value(), 1147);
    }

}
