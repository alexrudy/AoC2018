use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::ops::{Range, RangeInclusive};
use std::str::FromStr;

use failure::{Error, Fail};
use itertools::iproduct;
use lazy_static::lazy_static;
use regex::Regex;

use geometry::{BoundingBox, Direction, Point, Position};

pub(crate) fn main() -> Result<(), Error> {
    use crate::input_to_string;

    let scans = input_to_string(17)?
        .lines()
        .map(|l| l.parse::<Scan>())
        .collect::<Result<Vec<_>, _>>()?;

    let ground = Ground::from_scans(Point::new(500, 0), &scans);
    let mut system = WellSystem::new(ground, Water::new());
    system.fill();
    println!("Part 1: {}", system.wet());

    Ok(())
}

#[derive(Debug, Fail)]
enum ParseScanError {
    #[fail(display = "Failed to parse integer or range: {}", _0)]
    InvalidRange(String),

    #[fail(display = "Failed to parse scan: {}", _0)]
    InvalidScan(String),
}

#[derive(Debug, PartialEq)]
enum ScanCoordinate {
    Point(i32),
    Range(i32, i32),
}

impl ScanCoordinate {
    fn range(&self) -> RangeInclusive<i32> {
        match self {
            ScanCoordinate::Point(p) => (*p)..=(*p),
            ScanCoordinate::Range(s, e) => (*s)..=(*e),
        }
    }
}

impl FromStr for ScanCoordinate {
    type Err = ParseScanError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if let Ok(v) = s.parse::<i32>() {
            return Ok(ScanCoordinate::Point(v));
        }

        let parts = s
            .split("..")
            .map(|p| p.parse::<i32>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| ParseScanError::InvalidRange(s.to_string()))?;

        if parts.len() != 2 {
            return Err(ParseScanError::InvalidRange(s.to_owned()));
        }

        let start = parts[0];
        let end = parts[1];

        Ok(ScanCoordinate::Range(start, end))
    }
}

impl From<i32> for ScanCoordinate {
    fn from(d: i32) -> Self {
        ScanCoordinate::Point(d)
    }
}

impl From<Range<i32>> for ScanCoordinate {
    fn from(d: Range<i32>) -> Self {
        ScanCoordinate::Range(d.start, d.end)
    }
}

#[derive(Debug, PartialEq)]
struct Scan {
    x: ScanCoordinate,
    y: ScanCoordinate,
}

impl Scan {
    fn iter(&self) -> impl Iterator<Item = Point> {
        iproduct!(self.x.range(), self.y.range()).map(|(x, y)| Point::new(x, y))
    }
}

impl FromStr for Scan {
    type Err = ParseScanError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(x|y)=([\d]+), (x|y)=([\d]+\.\.[\d]+)").unwrap();
        }

        let cap = match RE.captures(s) {
            None => {
                return Err(ParseScanError::InvalidScan(s.to_string()));
            }
            Some(c) => c,
        };

        let label_a = &cap[1];
        let coord_a = cap[2].parse::<ScanCoordinate>()?;

        let label_b = &cap[3];
        let coord_b = cap[4].parse::<ScanCoordinate>()?;

        match (label_a, label_b) {
            ("x", "y") => Ok(Scan {
                x: coord_a,
                y: coord_b,
            }),
            ("y", "x") => Ok(Scan {
                x: coord_b,
                y: coord_a,
            }),
            _ => Err(ParseScanError::InvalidScan(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dirt {
    Clay,
    Sand,
}

impl fmt::Display for Dirt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Dirt::Sand => write!(f, "."),
            Dirt::Clay => write!(f, "#"),
        }
    }
}

#[derive(Debug)]
struct Ground {
    grid: BTreeSet<Point>,
    spring: Point,
    bbox: RefCell<Option<BoundingBox>>,
}

impl Default for Ground {
    fn default() -> Self {
        Self::new(Point::new(500, 0))
    }
}

impl Ground {
    fn new(spring: Point) -> Self {
        Self {
            grid: BTreeSet::new(),
            spring: spring,
            bbox: RefCell::from(None),
        }
    }

    fn from_scans(spring: Point, scans: &[Scan]) -> Self {
        let mut ground = Self::new(spring);
        for scan in scans {
            for point in scan.iter() {
                ground.insert(point, Dirt::Clay);
            }
        }
        ground
    }

    fn insert(&mut self, point: Point, dirt: Dirt) -> bool {
        let did_change = match dirt {
            Dirt::Clay => self.grid.insert(point),
            Dirt::Sand => self.grid.remove(&point),
        };
        if did_change {
            *self.bbox.borrow_mut() = None;
        }
        did_change
    }

    fn contains(&self, point: Point) -> bool {
        self.grid.contains(&point)
    }

    fn get(&self, point: Point) -> Dirt {
        if self.grid.contains(&point) {
            Dirt::Clay
        } else {
            Dirt::Sand
        }
    }

    fn bbox(&self) -> BoundingBox {
        {
            if let Some(bbox) = *self.bbox.borrow() {
                return bbox;
            }
        }

        let mut bbox = BoundingBox::empty();
        bbox.include(self.spring);
        for point in &self.grid {
            bbox.include(*point);
        }
        {
            *self.bbox.borrow_mut() = Some(bbox);
        }

        bbox
    }
}

impl fmt::Display for Ground {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bbox = self.bbox().horizontal_margin(1);
        for y in bbox.vertical() {
            for x in bbox.horizontal() {
                let point = Point::new(x, y);
                if point == self.spring {
                    write!(f, "+")?;
                } else {
                    write!(f, "{}", self.get(point))?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WaterFlow {
    Falling,
    Flowing,
    Standing,
}

impl fmt::Display for WaterFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WaterFlow::Falling => write!(f, "|"),
            WaterFlow::Flowing => write!(f, "-"),
            WaterFlow::Standing => write!(f, "~"),
        }
    }
}

struct Water(BTreeMap<Point, WaterFlow>);

impl Default for Water {
    fn default() -> Self {
        Self::new()
    }
}

impl Water {
    fn new() -> Self {
        Self(BTreeMap::new())
    }

    fn insert(&mut self, point: Point, flow: WaterFlow) -> bool {
        self.0.insert(point, flow).is_none()
    }

    fn contains(&self, point: Point) -> bool {
        self.0.contains_key(&point)
    }

    fn bbox(&self) -> BoundingBox {
        let mut bbox = BoundingBox::empty();
        for point in self.0.keys() {
            bbox.include(*point);
        }
        bbox
    }

    /// Is the water at this y-level yet?
    fn at_level(&self, level: Position) -> bool {
        let bbox = self.bbox();
        bbox.horizontal()
            .any(|x| self.0.contains_key(&Point::new(x, level)))
    }

    fn get(&self, point: Point) -> Option<&WaterFlow> {
        self.0.get(&point)
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

struct WellSystem {
    ground: Ground,
    water: Water,
}

impl Default for WellSystem {
    fn default() -> Self {
        Self {
            ground: Ground::default(),
            water: Water::default(),
        }
    }
}

enum FlowError {}

impl WellSystem {
    fn new(ground: Ground, water: Water) -> Self {
        Self { ground, water }
    }

    fn is_occupied(&self, point: Point) -> bool {
        self.ground.contains(point) || self.water.contains(point)
    }

    fn flow(&self, source: Point) -> Option<WaterFlow> {
        self.water.get(source).cloned()
    }

    fn fill_one(&mut self, source: Point) -> Vec<Point> {
        let bbox = self.ground.bbox().horizontal_margin(1);

        if !self.is_occupied(source.step(Direction::Down)) {
            self.water.insert(source, WaterFlow::Falling);
            return vec![source.step(Direction::Down)];
        }

        let mut water = BTreeSet::new();
        let mut flows = BTreeSet::new();

        for x in source.x..bbox.right() {
            let point = Point::new(x, source.y);

            if self.ground.contains(point) {
                break;
            }

            water.insert(point);

            // Is there any supporting ground? If not, stop, since
            // we will flow down here.
            if !self.is_occupied(point.step(Direction::Down)) {
                flows.insert(point.step(Direction::Down));
                break;
            }
        }

        for x in (bbox.left()..=source.x).rev() {
            let point = Point::new(x, source.y);

            if self.ground.contains(point) {
                break;
            }

            water.insert(point);

            // Is there any supporting ground? If not, stop, since
            // we will flow down here.
            if !self.is_occupied(point.step(Direction::Down)) {
                flows.insert(point.step(Direction::Down));
                break;
            }
        }

        if flows.is_empty() && self.is_occupied(source.step(Direction::Down)) {
            for w in &water {
                self.water.insert(*w, WaterFlow::Standing);
            }
            flows.insert(source.step(Direction::Up));
        } else {
            for w in &water {
                self.water.insert(*w, WaterFlow::Flowing);
            }
        };
        flows.into_iter().collect()
    }

    fn fill(&mut self) {
        let mut queue = VecDeque::new();
        queue.push_front(self.ground.spring.step(Direction::Down));

        let bbox = self.ground.bbox().horizontal_margin(1);

        while !queue.is_empty() {
            let point = queue.pop_front().expect("Queue is empty");

            let falling = self.fill_one(point);

            for flow in &falling {
                if bbox.contains(*flow) {
                    queue.push_back(*flow);
                }
            }
        }
    }

    fn wet(&self) -> usize {
        self.water.len()
    }

    fn bbox(&self) -> BoundingBox {
        self.ground.bbox().union(&self.water.bbox())
    }
}

impl fmt::Display for WellSystem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bbox = self.bbox().horizontal_margin(1);
        for y in bbox.vertical() {
            for x in bbox.horizontal() {
                let point = Point::new(x, y);
                if point == self.ground.spring {
                    write!(f, "+")?;
                } else if let Some(flow) = self.flow(point) {
                    write!(f, "{}", flow)?;
                } else {
                    write!(f, "{}", self.ground.get(point))?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn scan() {
        let s1: Scan = "x=495, y=2..7".parse().unwrap();
        assert_eq!(
            s1,
            Scan {
                x: 495.into(),
                y: (2..7).into()
            }
        );

        let s2: Scan = "y=7, x=495..501".parse().unwrap();
        assert_eq!(
            s2,
            Scan {
                x: (495..501).into(),
                y: 7.into()
            }
        );
    }

    #[test]
    fn ground() {
        let ground_diagram = "......+.......
............#.
.#..#.......#.
.#..#..#......
.#..#..#......
.#.....#......
.#.....#......
.#######......
..............
..............
....#.....#...
....#.....#...
....#.....#...
....#######...
";

        assert_eq!(example_ground().to_string(), ground_diagram);
    }

    fn example_ground() -> Ground {
        let scans = "x=495, y=2..7
y=7, x=495..501
x=501, y=3..7
x=498, y=2..4
x=506, y=1..2
x=498, y=10..13
x=504, y=10..13
y=13, x=498..504"
            .lines()
            .map(|l| l.parse::<Scan>())
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        Ground::from_scans(Point::new(500, 0), &scans)
    }

    #[test]
    fn flow() {
        let ground = example_ground();

        let mut system = WellSystem::new(ground, Water::new());
        assert_eq!(
            system.fill_one(system.ground.spring),
            vec![Point::new(500, 1)]
        );

        assert_eq!(system.flow(system.ground.spring), Some(WaterFlow::Falling));

        assert_eq!(
            system.fill_one(Point::new(500, 1)),
            vec![Point::new(500, 2)]
        );
        assert_eq!(
            system.fill_one(Point::new(500, 2)),
            vec![Point::new(500, 3)]
        );
        assert_eq!(
            system.fill_one(Point::new(500, 3)),
            vec![Point::new(500, 4)]
        );

        assert_eq!(
            system.fill_one(Point::new(500, 4)),
            vec![Point::new(500, 5)]
        );

        assert_eq!(
            system.fill_one(Point::new(500, 5)),
            vec![Point::new(500, 6)]
        );
        assert_eq!(system.flow(Point::new(500, 5)), Some(WaterFlow::Falling));

        assert_eq!(
            system.fill_one(Point::new(500, 6)),
            vec![Point::new(500, 5)]
        );
        eprintln!("{}", system);
        assert_eq!(system.flow(Point::new(500, 6)), Some(WaterFlow::Standing));
        assert_eq!(system.flow(Point::new(500, 5)), Some(WaterFlow::Falling));
        assert_eq!(system.flow(Point::new(500, 4)), Some(WaterFlow::Falling));

        assert_eq!(
            system.fill_one(Point::new(500, 5)),
            vec![Point::new(500, 4)]
        );
        assert_eq!(system.flow(Point::new(500, 6)), Some(WaterFlow::Standing));
        assert_eq!(system.flow(Point::new(500, 5)), Some(WaterFlow::Standing));
        assert_eq!(system.flow(Point::new(500, 4)), Some(WaterFlow::Falling));
    }

    #[test]
    fn example_part1() {
        let result = "......+.......
......|.....#.
.#..#----...#.
.#..#~~#|.....
.#..#~~#|.....
.#~~~~~#|.....
.#~~~~~#|.....
.#######|.....
........|.....
...---------..
...|#~~~~~#|..
...|#~~~~~#|..
...|#~~~~~#|..
...|#######|..
";

        let ground = example_ground();
        let mut system = WellSystem::new(ground, Water::new());

        system.fill();
        assert_eq!(&system.to_string(), result);

        assert_eq!(system.wet(), 57);
    }

}
