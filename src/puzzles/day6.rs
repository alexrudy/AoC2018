use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::BufRead;
use std::str::FromStr;

use regex::Regex;

type Result<T> = ::std::result::Result<T, Box<Error>>;

#[derive(Debug, PartialEq, Clone, Copy, Eq, PartialOrd, Ord, Hash)]
struct Point {
    x: i32,
    y: i32,
}

impl FromStr for Point {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?P<x>[\d]+),\s*(?P<y>[\d]+)").unwrap();
        };

        let cap = match RE.captures(s) {
            None => return err!("Can't parse point {}", s),
            Some(c) => c,
        };

        Ok(Self {
            x: cap["x"].parse()?,
            y: cap["y"].parse()?,
        })
    }
}

impl Point {
    fn distance(self, other: Point) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, PartialOrd, Ord, Hash)]
struct BBox {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl BBox {
    fn from_points(points: &[Point]) -> Self {
        let x_max = points.iter().map(|p| p.x).max().unwrap() + 1;
        let y_max = points.iter().map(|p| p.y).max().unwrap() + 1;
        let x_min = points.iter().map(|p| p.x).min().unwrap() - 1;
        let y_min = points.iter().map(|p| p.y).min().unwrap() - 1;

        Self {
            x: x_min,
            y: y_min,
            w: x_max - x_min,
            h: y_max - y_min,
        }
    }

    fn iter(&self) -> BBoxIterator {
        BBoxIterator {
            bbox: &self,
            px: 0,
            py: 0,
        }
    }

    fn edge(&self, point: Point) -> bool {
        point.x == self.x
            || point.x == (self.x + self.w)
            || point.y == self.y
            || point.y == (self.y + self.h)
    }
}

struct BBoxIterator<'b> {
    bbox: &'b BBox,
    px: i32,
    py: i32,
}

impl<'b> Iterator for BBoxIterator<'b> {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.px == self.bbox.w {
            self.px = 0;
            self.py += 1;
        }

        if self.py >= self.bbox.h {
            return None;
        }

        let result = Some(Point {
            x: self.px + self.bbox.x,
            y: self.py + self.bbox.y,
        });
        self.px += 1;
        result
    }
}

#[derive(Debug, PartialEq)]
struct Grid {
    bbox: BBox,
    counter: HashMap<Point, i32>,
    infinite: HashSet<Point>,
}

impl Grid {
    fn from_bbox(bbox: BBox) -> Self {
        Self {
            bbox: bbox,
            counter: HashMap::new(),
            infinite: HashSet::new(),
        }
    }

    fn check(&mut self, location: Point, points: &[Point]) {
        if let Some(pt) = closest(location, points) {
            *self.counter.entry(pt).or_insert(0) += 1;

            if self.bbox.edge(location) {
                self.infinite.insert(pt);
            }
        }
    }

    fn largest_area(&self) -> i32 {
        *self
            .counter
            .iter()
            .filter(|(p, _)| !self.infinite.contains(&p))
            .map(|(_, c)| c)
            .max()
            .unwrap()
    }
}

fn vornoi_largest_area(points: &[Point]) -> i32 {
    let bbox = BBox::from_points(points);
    let mut grid = Grid::from_bbox(bbox);

    for location in bbox.iter() {
        grid.check(location, points);
    }

    grid.largest_area()
}

fn protected_area(points: &[Point], distance_limit: i32) -> usize {
    let bbox = BBox::from_points(points);

    bbox.iter()
        .filter(|location| {
            points.iter().map(|p| location.distance(*p)).sum::<i32>() < distance_limit
        })
        .count()
}

fn closest(location: Point, points: &[Point]) -> Option<Point> {
    let mut cl = points[0];
    let mut unique = true;

    for pt in points.iter().skip(1) {
        if location.distance(*pt) < location.distance(cl) {
            cl = *pt;
            unique = true;
        } else if location.distance(*pt) == location.distance(cl) {
            unique = false;
        }
    }

    if !unique {
        None
    } else {
        Some(cl)
    }
}

fn get_input() -> Result<Vec<Point>> {
    use crate::input;

    Ok(input(6)?
        .lines()
        .map(|l| l.map_err(Box::<Error>::from).and_then(|s| s.parse()))
        .collect::<Result<Vec<Point>>>()?)
}

pub(crate) fn main() -> Result<()> {
    let points = get_input()?;

    println!("Part 1: {}", vornoi_largest_area(&points));
    println!("Part 2: {}", protected_area(&points, 10_000));
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    const POINTS: &str = "1, 1
1, 6
8, 3
3, 4
5, 5
8, 9";

    fn points() -> Vec<Point> {
        POINTS
            .lines()
            .map(|s| s.parse::<Point>().unwrap())
            .collect()
    }

    #[test]
    fn example_part1() {
        let points: Vec<Point> = points();

        assert_eq!(points[0], Point { x: 1, y: 1 });

        assert_eq!(points[0].distance(Point { x: 0, y: 3 }), 3);

        assert_eq!(
            closest(Point { x: 0, y: 3 }, &points),
            Some(Point { x: 1, y: 1 })
        );

        assert_eq!(vornoi_largest_area(&points), 17);
    }

    #[test]
    fn answer_part1() {
        let points = get_input().unwrap();
        assert_eq!(vornoi_largest_area(&points), 3722);
    }

    #[test]
    fn example_part2() {
        let points: Vec<Point> = points();

        assert_eq!(protected_area(&points, 32), 16);
    }

    #[test]
    fn answer_part2() {
        let points = get_input().unwrap();

        assert_eq!(protected_area(&points, 10_000), 44634);
    }
}
