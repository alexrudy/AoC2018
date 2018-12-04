use std::collections::HashMap;
use std::error::Error;
use std::io::BufRead;
use std::str::FromStr;

use regex::Regex;

type Coordinate = u32;

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<Error>::from(format!($($tt)*))) }
}

type Grid = HashMap<(Coordinate, Coordinate), usize>;

fn grid<'c, C>(claims: C) -> Grid
where
    C: Iterator<Item = &'c Claim>,
{
    let mut grid = Grid::new();

    for claim in claims {
        for point in claim.points() {
            *grid.entry(point).or_insert(0) += 1;
        }
    }
    grid
}

fn overlap(grid: &Grid) -> usize {
    grid.values().filter(|&&c| c > 1).count()
}

fn no_overlap(claims: &[Claim], grid: &Grid) -> Result<usize, Box<Error>> {
    for claim in claims {
        if claim.points().all(|ref p| grid[p] == 1) {
            return Ok(claim.id);
        }
    }
    err!("No claim has zero overlaps!")
}

pub(crate) fn main() -> Result<(), Box<Error>> {
    use crate::input;

    let claims = input(3)?
        .lines()
        .map(|lr| {
            lr.map_err(|err| err.into())
                .and_then(|l| l.parse::<Claim>())
        })
        .collect::<Result<Vec<Claim>, _>>()?;
    let grid = grid(claims.iter());

    println!("Part 1: {}", overlap(&grid));
    println!("Part 2: #{}", no_overlap(&claims, &grid)?);

    Ok(())
}

#[derive(Debug, PartialEq)]
struct Claim {
    id: usize,
    x: Coordinate,
    y: Coordinate,
    w: Coordinate,
    h: Coordinate,
}

impl Claim {
    fn points(&self) -> ClaimPoints {
        ClaimPoints {
            claim: self,
            px: 0,
            py: 0,
        }
    }
}

struct ClaimPoints<'c> {
    claim: &'c Claim,
    px: Coordinate,
    py: Coordinate,
}

impl<'c> Iterator for ClaimPoints<'c> {
    type Item = (Coordinate, Coordinate);

    fn next(&mut self) -> Option<Self::Item> {
        if self.px == self.claim.w {
            self.px = 0;
            self.py += 1;
        }

        if self.py >= self.claim.h {
            return None;
        }

        let result = Some((self.px + self.claim.x, self.py + self.claim.y));
        self.px += 1;
        result
    }
}

impl FromStr for Claim {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?x)
                \#(?P<id>[\d]+)
                \s*@\s*
                (?P<x>[\d]+)
                \s*,\s*
                (?P<y>[\d]+)
                \s*:\s*
                (?P<w>[\d]+)
                \s*x\s*
                (?P<h>[\d]+)
                "
            )
            .unwrap();
        };

        let cap = match RE.captures(s) {
            None => return err!("Can't match claim {}", s),
            Some(cap) => cap,
        };

        Ok(Self {
            id: cap["id"].parse()?,
            x: cap["x"].parse()?,
            y: cap["y"].parse()?,
            w: cap["w"].parse()?,
            h: cap["h"].parse()?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example_claim() {
        let claim: Claim = "#123 @ 3,2: 5x4".parse().unwrap();

        assert_eq!(
            claim,
            Claim {
                id: 123,
                x: 3,
                y: 2,
                w: 5,
                h: 4
            }
        );

        let points: Vec<(Coordinate, Coordinate)> = claim.points().collect();

        assert_eq!(
            points,
            vec![
                (3, 2),
                (4, 2),
                (5, 2),
                (6, 2),
                (7, 2),
                (3, 3),
                (4, 3),
                (5, 3),
                (6, 3),
                (7, 3),
                (3, 4),
                (4, 4),
                (5, 4),
                (6, 4),
                (7, 4),
                (3, 5),
                (4, 5),
                (5, 5),
                (6, 5),
                (7, 5)
            ]
        )
    }

    #[test]
    fn example_part1() {
        let claims: Vec<Claim> = "#1 @ 1,3: 4x4
                #2 @ 3,1: 4x4
                #3 @ 5,5: 2x2"
            .split('\n')
            .map(|s| s.trim().parse())
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let grid = grid(claims.iter());

        assert_eq!(overlap(&grid), 4);
    }
}
