#![feature(try_from)]

use std::cell::RefCell;
use std::cmp;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashSet};
use std::fmt;
use std::ops::{Range, RangeInclusive};
use std::str::FromStr;

use failure::Fail;
use itertools::iproduct;
use lazy_static::lazy_static;
use regex::Regex;

use geometry::{BoundingBox, Direction, Point, Position};

type WSet = BTreeSet<Point>;
type WXMap = BTreeMap<Position, WaterFlow>;
type WMap = BTreeMap<Position, WXMap>;

pub mod views;

#[derive(Debug, Fail)]
pub enum ParseScanError {
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
pub struct Scan {
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

#[derive(Debug, Clone)]
pub struct Ground {
    grid: BTreeMap<Position, HashSet<Position>>,
    spring: Point,
    bbox: RefCell<Option<BoundingBox>>,
}

impl Default for Ground {
    fn default() -> Self {
        Self::new(Point::new(500, 0))
    }
}

impl Ground {
    pub fn new(spring: Point) -> Self {
        Self {
            grid: BTreeMap::new(),
            spring: spring,
            bbox: RefCell::from(None),
        }
    }

    pub fn from_scans(spring: Point, scans: &[Scan]) -> Self {
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
            Dirt::Clay => self
                .grid
                .entry(point.y)
                .or_insert_with(HashSet::new)
                .insert(point.x),
            Dirt::Sand => self
                .grid
                .entry(point.y)
                .or_insert_with(HashSet::new)
                .remove(&point.x),
        };
        if did_change {
            *self.bbox.borrow_mut() = None;
        }
        did_change
    }

    fn contains(&self, point: Point) -> bool {
        self.grid
            .get(&point.y)
            .map_or(false, |s| s.contains(&point.x))
    }

    fn get(&self, point: Point) -> Dirt {
        if self.contains(point) {
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

        for (y, xs) in &self.grid {
            for x in xs {
                bbox.include(Point::new(*x, *y))
            }
        }
        {
            *self.bbox.borrow_mut() = Some(bbox);
        }

        bbox
    }
}

impl fmt::Display for Ground {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut bbox = self.bbox().horizontal_margin(1);
        bbox.include(self.spring);

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

#[derive(Debug, Fail)]
pub enum ParseGroundError {
    #[fail(display = "Invalid ground: {}", _0)]
    InvalidGround(String),
}

impl FromStr for Ground {
    type Err = ParseGroundError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ground = Self::default();
        for (y, line) in s.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                let point = Point::new(x as i32, y as i32);
                match c {
                    '#' => ground.insert(point, Dirt::Clay),
                    '.' => ground.insert(point, Dirt::Sand),
                    ' ' => ground.insert(point, Dirt::Sand),
                    '+' => {
                        ground.spring = point;
                        true
                    }
                    c => return Err(ParseGroundError::InvalidGround(format!("{}", c))),
                };
            }
        }
        Ok(ground)
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

#[derive(Debug, Clone)]
pub struct Water(WMap);

impl Default for Water {
    fn default() -> Self {
        Self::new()
    }
}

impl Water {
    pub fn new() -> Self {
        Self(WMap::new())
    }

    fn insert(&mut self, point: Point, flow: WaterFlow) -> bool {
        self.0
            .entry(point.y)
            .or_insert_with(WXMap::new)
            .insert(point.x, flow)
            .is_none()
    }

    fn bbox(&self) -> BoundingBox {
        let mut bbox = BoundingBox::empty();
        for (y, xs) in &self.0 {
            for x in xs.keys() {
                bbox.include(Point::new(*x, *y));
            }
        }
        bbox
    }

    fn level_bbox(&self) -> Option<BoundingBox> {
        let y = self.0.keys().max()?;
        let mut bbox = BoundingBox::empty();
        for x in self.0.get(y)?.keys() {
            bbox.include(Point::new(*x, *y));
        }
        Some(bbox)
    }

    fn get(&self, point: Point) -> Option<&WaterFlow> {
        self.0.get(&point.y).and_then(|xs| xs.get(&point.x))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaterLevel {
    pub flows: Vec<Point>,
    pub covered: Vec<Point>,
    mode: WaterFlow,
}

impl WaterLevel {
    fn falling(source: Point) -> Self {
        Self {
            flows: vec![source.step(Direction::Down)],
            covered: vec![source],
            mode: WaterFlow::Falling,
        }
    }

    fn standing(source: Point, covered: Vec<Point>) -> Self {
        Self {
            flows: vec![source.step(Direction::Up)],
            covered: covered,
            mode: WaterFlow::Standing,
        }
    }

    fn surrounded() -> Self {
        Self {
            flows: Vec::new(),
            covered: Vec::new(),
            mode: WaterFlow::Standing,
        }
    }

    pub fn bbox(&self) -> BoundingBox {
        let mut bbox = BoundingBox::empty();
        for p in &self.covered {
            bbox.include(*p);
        }
        bbox
    }
}

#[derive(Debug, Clone)]
pub struct WellSystem {
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

impl WellSystem {
    pub fn new(ground: Ground, water: Water) -> Self {
        Self { ground, water }
    }

    // fn is_occupied(&self, point: Point) -> bool {
    //     self.ground.contains(point) || self.water.contains(point)
    // }

    fn flow(&self, source: Point) -> Option<WaterFlow> {
        self.water.get(source).cloned()
    }

    fn fill_one(&self, source: Point) -> WaterLevel {
        if self
            .flow(source)
            .map_or(false, |f| f == WaterFlow::Standing)
        {
            return WaterLevel::standing(source, Vec::new());
        }

        let bbox = self.ground.bbox().horizontal_margin(2);

        let below = source.step(Direction::Down);
        let above = source.step(Direction::Up);

        let ground_below = self.ground.grid.get(&below.y);
        let water_below = self.water.0.get(&below.y);

        let below_is_occupied = |x: i32| {
            ground_below.map_or(false, |xs| xs.contains(&x))
                || water_below
                    .and_then(|xs| xs.get(&x))
                    .map_or(false, |&f| f == WaterFlow::Standing)
        };

        // Fall if there is nothing below us!
        if !below_is_occupied(below.x) {
            return WaterLevel::falling(source);
        }

        let mut water = WSet::new();
        water.insert(source);

        let mut flows = WSet::new();

        let level = self.ground.grid.get(&source.y);

        for x in (source.x..bbox.right()).skip(1) {
            let point = Point::new(x, source.y);

            if level.map_or(false, |s| s.contains(&x)) {
                break;
            }

            water.insert(point);

            // Is there any supporting ground? If not, stop, since
            // we will flow down here.
            if !below_is_occupied(point.step(Direction::Down).x) {
                flows.insert(point.step(Direction::Down));
                break;
            }
        }

        for x in (bbox.left()..=source.x).rev() {
            let point = Point::new(x, source.y);

            if level.map_or(false, |s| s.contains(&x)) {
                break;
            }

            water.insert(point);

            // Is there any supporting ground? If not, stop, since
            // we will flow down here.
            if !below_is_occupied(point.step(Direction::Down).x) {
                flows.insert(point.step(Direction::Down));
                break;
            }
        }

        if flows.is_empty() && below_is_occupied(source.step(Direction::Down).x) {
            if self.flow(above).map_or(false, |f| f == WaterFlow::Standing) {
                WaterLevel::surrounded()
            } else {
                WaterLevel::standing(source, water.into_iter().collect())
            }
        } else {
            WaterLevel {
                flows: flows.into_iter().collect(),
                covered: water.into_iter().collect(),
                mode: WaterFlow::Flowing,
            }
        }
    }

    pub fn fill(&mut self) -> WellIterator {
        WellIterator::new(self)
    }

    pub fn insert(&mut self, water: &WaterLevel) {
        let mode = water.mode;
        for w in &water.covered {
            self.water.insert(*w, mode);
        }
    }

    pub fn wet(&self) -> usize {
        let bbox = self.ground.bbox().horizontal_margin(2);
        let mut w = 0;
        for (y, xs) in &self.water.0 {
            for x in xs.keys() {
                if bbox.contains(Point::new(*x, *y)) {
                    w += 1
                }
            }
        }
        w
    }

    pub fn retained(&self) -> usize {
        let bbox = self.ground.bbox().horizontal_margin(2);
        let mut w = 0;
        for (y, xs) in &self.water.0 {
            for x in xs.keys() {
                let point = Point::new(*x, *y);
                if bbox.contains(point)
                    && self.flow(point).map_or(false, |f| f == WaterFlow::Standing)
                {
                    w += 1
                }
            }
        }
        w
    }

    pub fn bbox(&self) -> BoundingBox {
        self.ground.bbox().union(&self.water.bbox())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct QPoint(Point);

impl cmp::Ord for QPoint {
    fn cmp(&self, other: &QPoint) -> cmp::Ordering {
        self.0.cmp(&other.0).reverse()
    }
}

impl cmp::PartialOrd for QPoint {
    fn partial_cmp(&self, other: &QPoint) -> Option<cmp::Ordering> {
        Some(self.0.cmp(&other.0).reverse())
    }
}

impl From<Point> for QPoint {
    fn from(p: Point) -> QPoint {
        QPoint(p)
    }
}

impl From<QPoint> for Point {
    fn from(q: QPoint) -> Point {
        q.0
    }
}

#[derive(Debug)]
pub struct WellIterator<'a> {
    pub(crate) queue: BinaryHeap<QPoint>,
    pending: BTreeSet<Point>,
    system: &'a mut WellSystem,
    bbox: BoundingBox,
}

impl<'a> WellIterator<'a> {
    fn new(system: &'a mut WellSystem) -> Self {
        let down = system.ground.spring.step(Direction::Down);
        let mut queue = BinaryHeap::new();
        queue.push(down.into());
        let mut pending = BTreeSet::new();
        pending.insert(down);
        let mut bbox = system.ground.bbox().horizontal_margin(1);
        bbox.include(system.ground.spring);

        Self {
            queue,
            system,
            bbox,
            pending,
        }
    }
}

impl<'a> Iterator for WellIterator<'a> {
    type Item = WaterLevel;

    fn next(&mut self) -> Option<Self::Item> {
        let point = Point::from(self.queue.pop()?);
        self.pending.remove(&point);

        let falling = self.system.fill_one(point);
        self.system.insert(&falling);

        for flow in &falling.flows {
            if self.bbox.contains(*flow) && !self.pending.contains(flow) {
                self.pending.insert(*flow);
                self.queue.push(QPoint::from(*flow));
            }
        }
        Some(falling)
    }
}

impl fmt::Display for WellSystem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut bbox = self.bbox().horizontal_margin(1);
        bbox.include(self.ground.spring);

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
        let ground_diagram = include_str!("../examples/example_part1.empty.txt");

        assert_eq!(example_ground().to_string(), ground_diagram);
    }

    fn example_ground() -> Ground {
        let scans = include_str!("../example.txt")
            .lines()
            .map(|l| l.parse::<Scan>())
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        Ground::from_scans(Point::new(500, 0), &scans)
    }

    macro_rules! check_wl {
        ($sys:expr, $origin:expr, $wl:expr) => {
            let mut gwl = $sys.fill_one($origin);
            gwl.covered.sort();
            gwl.flows.sort();
            $sys.insert(&gwl);
            assert_eq!(gwl, $wl);
        };
    }

    #[test]
    fn flow() {
        let ground = example_ground();

        let mut system = WellSystem::new(ground, Water::new());

        check_wl!(
            system,
            system.ground.spring,
            WaterLevel::falling(Point::new(500, 0))
        );

        check_wl!(
            system,
            Point::new(500, 1),
            WaterLevel::falling(Point::new(500, 1))
        );

        check_wl!(
            system,
            Point::new(500, 2),
            WaterLevel::falling(Point::new(500, 2))
        );

        check_wl!(
            system,
            Point::new(500, 3),
            WaterLevel::falling(Point::new(500, 3))
        );

        check_wl!(
            system,
            Point::new(500, 4),
            WaterLevel::falling(Point::new(500, 4))
        );

        check_wl!(
            system,
            Point::new(500, 5),
            WaterLevel::falling(Point::new(500, 5))
        );

        check_wl!(
            system,
            Point::new(500, 6),
            WaterLevel::standing(
                Point::new(500, 6),
                Scan::from_str("y=6, x=496..500").unwrap().iter().collect()
            )
        );

        check_wl!(
            system,
            Point::new(500, 5),
            WaterLevel::standing(
                Point::new(500, 5),
                Scan::from_str("y=5, x=496..500").unwrap().iter().collect()
            )
        );
    }

    #[test]
    fn example_part1and2() {
        let result = include_str!("../examples/example_part1.filled.txt");

        let ground = example_ground();
        let mut system = WellSystem::new(ground, Water::new());

        let _ = system.fill().last().unwrap();
        assert_eq!(&system.to_string(), result);

        assert_eq!(system.wet(), 57);
        assert_eq!(system.retained(), 29);
    }

    #[test]
    fn double_fill() {
        let ground: Ground = include_str!("../examples/doublefill.txt").parse().unwrap();
        let mut system = WellSystem::new(ground, Water::new());

        eprintln!("{}", system);
        let wl = system.fill().last().expect("At 1 tick happens");
        eprintln!("{}", system);
        assert_eq!(wl, WaterLevel::falling(Point::new(3, 13)))
    }

    #[test]
    fn rejoin_water() {
        let ground: Ground = include_str!("../examples/rejoiner.txt").parse().unwrap();
        let mut system = WellSystem::new(ground, Water::new());

        eprintln!("{}", system);
        let wl = system.fill().last().expect("At 1 tick goes by.");
        eprintln!("{}", system);
        assert_eq!(wl, WaterLevel::falling(Point::new(13, 17)))
    }

    #[test]
    fn twoflow() {
        let ground: Ground = include_str!("../examples/twoflow.txt").parse().unwrap();
        let mut system = WellSystem::new(ground, Water::new());

        eprintln!("{}", system);
        let wl = system.fill().last().expect("At 1 tick goes by.");
        eprintln!("{}", system);
        assert_eq!(wl, WaterLevel::falling(Point::new(12, 17)))
    }

    #[test]
    fn tubeflow() {
        let ground: Ground = include_str!("../examples/tubeflow.txt").parse().unwrap();
        let mut system = WellSystem::new(ground, Water::new());

        eprintln!("{}", system);
        let wl = system.fill().last().expect("At 1 tick goes by.");
        eprintln!("{}", system);
        assert_eq!(wl, WaterLevel::falling(Point::new(15, 19)))
    }
}
