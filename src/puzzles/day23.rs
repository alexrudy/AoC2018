use std::num::ParseIntError;
use std::str::FromStr;

use failure::{Error, Fail};
use lazy_static::lazy_static;
use regex::Regex;

pub(crate) fn main() -> Result<(), Error> {
    Ok(())
}

type Position = i32;

#[derive(Debug, PartialEq, Eq, Clone)]
struct Point3D {
    x: Position,
    y: Position,
    z: Position,
}

impl Point3D {
    fn distance(&self, other: &Self) -> Position {
        (self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Nanobot {
    location: Point3D,
    r: Position,
}

impl Nanobot {
    fn distance(&self, other: &Self) -> Position {
        self.location.distance(&other.location)
    }
}

#[derive(Debug, Fail)]
enum ParseNanobotError {
    #[fail(display = "Failed to match: {}", _0)]
    PatternMatchFailed(String),

    #[fail(display = "Invalid number")]
    InvalidNumber,
}

impl From<ParseIntError> for ParseNanobotError {
    fn from(_: ParseIntError) -> Self {
        ParseNanobotError::InvalidNumber
    }
}

impl FromStr for Nanobot {
    type Err = ParseNanobotError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"pos=<(?P<x>\d+),(?P<y>\d+),(?P<z>\d+)>,\s+r=(?P<r>\d+)").unwrap();
        }

        let cap = match RE.captures(s) {
            Some(c) => c,
            None => return Err(ParseNanobotError::PatternMatchFailed(s.to_string())),
        };

        let location = Point3D {
            x: cap["x"].parse()?,
            y: cap["y"].parse()?,
            z: cap["z"].parse()?,
        };

        Ok(Self {
            location,
            r: cap["r"].parse()?,
        })
    }
}

#[derive(Debug)]
struct KDNode {
    location: Nanobot,
    left: Option<Box<KDNode>>,
    right: Option<Box<KDNode>>,
}

impl KDNode {
    fn build(points: &[&Nanobot], depth: usize) -> KDNode {
        // Find the midpoint of the elements
        let mut points: Vec<&Nanobot> = points.to_vec();
        points.sort_by_key(|&p| match depth % 3 {
            0 => p.location.x,
            1 => p.location.y,
            2 => p.location.z,
            _ => unreachable!(),
        });

        let midpoint = points.len() / 2;
        let location = points[midpoint];

        // Make the child nodes
        let left = if midpoint > 0 {
            Some(Box::from(KDNode::build(&points[0..midpoint], depth + 1)))
        } else {
            None
        };

        let right = if midpoint < points.len() {
            Some(Box::from(KDNode::build(&points[midpoint..], depth + 1)))
        } else {
            None
        };

        // Make the node.
        KDNode {
            location: location.clone(),
            left,
            right,
        }
    }
}

#[derive(Debug)]
struct KDTree {
    root: KDNode,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn nanobot() {
        let na: Nanobot = "pos=<0,0,0>, r=4".parse().unwrap();
        let nb: Nanobot = "pos=<1,0,0>, r=1".parse().unwrap();

        assert_eq!(na.distance(&nb), 1);
    }
}
