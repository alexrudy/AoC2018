use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::BufRead;
use std::str::FromStr;

use geometry;

use regex::Regex;

type Result<T> = ::std::result::Result<T, Box<Error>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Point(geometry::Point);

impl Point {
    #[cfg(test)]
    fn new(x: i32, y: i32) -> Self {
        Point(geometry::Point::new(x, y))
    }
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

        Ok(Point(geometry::Point::new(
            cap["x"].parse()?,
            cap["y"].parse()?,
        )))
    }
}

impl Point {
    fn distance(self, other: Point) -> i32 {
        self.0.manhattan_distance(other.0)
    }
}

impl From<Point> for geometry::Point {
    fn from(p: Point) -> Self {
        p.0
    }
}

impl From<geometry::Point> for Point {
    fn from(p: geometry::Point) -> Self {
        Point(p)
    }
}

#[derive(Debug, PartialEq)]
struct Grid {
    bbox: geometry::BoundingBox,
    counter: HashMap<Point, i32>,
    infinite: HashSet<Point>,
}

impl Grid {
    fn from_bbox(bbox: geometry::BoundingBox) -> Self {
        Self {
            bbox: bbox,
            counter: HashMap::new(),
            infinite: HashSet::new(),
        }
    }

    fn check(&mut self, location: Point, points: &[Point]) {
        if let Some(pt) = closest(location, points) {
            *self.counter.entry(pt).or_insert(0) += 1;

            if self.bbox.is_edge(location.into()) {
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
    let bbox = geometry::BoundingBox::from_points(points.iter().map(|&p| p.into()));
    let mut grid = Grid::from_bbox(bbox);

    for location in bbox.points() {
        grid.check(location.into(), points);
    }

    grid.largest_area()
}

fn protected_area(points: &[Point], distance_limit: i32) -> usize {
    let bbox = geometry::BoundingBox::from_points(points.iter().map(|&p| p.into()));

    bbox.points()
        .filter(|&location| {
            let glocation: Point = location.into();
            points.iter().map(|&p| glocation.distance(p)).sum::<i32>() < distance_limit
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

        assert_eq!(points[0], Point::new(1, 1));

        assert_eq!(points[0].distance(Point::new(0, 3)), 3);

        assert_eq!(closest(Point::new(0, 3), &points), Some(Point::new(1, 1)));

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
