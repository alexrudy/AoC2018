#![allow(dead_code)]

use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::convert::TryFrom;
use std::fmt;
use std::iter::FromIterator;
use std::num::TryFromIntError;
use std::str::FromStr;

use failure::Error;

use crate::geometry::{BoundingBox, Direction, Point, Position};
use crate::sprite::{Health, ParseSpeciesError, Species, Sprite, SpriteBuilder, SpriteStatus};

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

impl Grid {
    fn new() -> Self {
        Grid(HashSet::new())
    }

    fn insert(&mut self, point: Point, tile: Tile) -> bool {
        match tile {
            Tile::Empty => self.0.insert(point),
            Tile::Wall => self.0.remove(&point),
        }
    }

    fn get(&self, point: Point) -> Tile {
        if self.0.contains(&point) {
            Tile::Empty
        } else {
            Tile::Wall
        }
    }

    fn bbox(&self) -> BoundingBox {
        let mut bbox = BoundingBox::empty();
        for point in &self.0 {
            bbox.include(*point);
        }
        bbox
    }
}

#[derive(Debug, Clone)]
pub struct Sprites {
    sprites: HashMap<Point, Sprite>,
}

impl Sprites {
    fn new() -> Self {
        Self {
            sprites: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.sprites.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sprites.is_empty()
    }

    pub fn place(&mut self, sprite: Sprite) {
        self.sprites.insert(sprite.position(), sprite);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Sprite> {
        self.sprites.values()
    }

    pub fn positions(&self) -> impl Iterator<Item = &Point> {
        self.sprites.keys()
    }

    pub fn get(&self, point: Point) -> Option<&Sprite> {
        self.sprites.get(&point)
    }

    fn peek(&self) -> Option<&Sprite> {
        let positions: BinaryHeap<Point> = self.positions().cloned().collect();
        positions.peek().and_then(|p| self.get(*p))
    }

    fn step(&mut self, point: Point, direction: Direction) {
        let mut sprite = self.sprites.remove(&point).unwrap();
        sprite.step(direction);
        self.place(sprite);
    }

    fn attack(&mut self, aggressor: Point, target: Point) -> SpriteStatus {
        let power = self.sprites[&aggressor].attack();
        let victim = self.sprites.get_mut(&target).unwrap();
        let result = victim.wound(power);

        // Remove corpses from the battlefield.
        if let SpriteStatus::Dead = result {
            self.sprites.remove(&target);
        };
        result
    }

    fn bbox(&self) -> BoundingBox {
        let mut bbox = BoundingBox::empty();
        for position in self.sprites.keys() {
            bbox.include(*position);
        }
        bbox
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpritePath {
    destination: Point,
    direction: Direction,
    distance: Position,
}

impl SpritePath {
    fn paths(point: Point) -> Vec<SpritePath> {
        let mut paths = Vec::with_capacity(4);
        for direction in Direction::all() {
            paths.push(SpritePath {
                destination: point.step(direction),
                direction: direction,
                distance: 1,
            })
        }
        paths
    }

    fn extend(&self, direction: Direction) -> Self {
        Self {
            destination: self.destination.step(direction),
            direction: self.direction,
            distance: self.distance + 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoundOutcome {
    NoAction,
    CombatOnly,
    Casualty,
    Movement,
    MidRoundVictory(Species),
    Victory(Species),
}

impl RoundOutcome {
    fn combat(self) -> Self {
        match self {
            RoundOutcome::NoAction => RoundOutcome::CombatOnly,
            others => others,
        }
    }

    fn movement(self) -> Self {
        match self {
            RoundOutcome::NoAction => RoundOutcome::Movement,
            RoundOutcome::CombatOnly => RoundOutcome::Movement,
            others => others,
        }
    }

    fn casualty(self) -> Self {
        match self {
            RoundOutcome::CombatOnly => RoundOutcome::Casualty,
            RoundOutcome::NoAction => RoundOutcome::Casualty,
            others => others,
        }
    }

    fn is_finished(self) -> bool {
        match self {
            RoundOutcome::CombatOnly => false,
            RoundOutcome::Movement => false,
            RoundOutcome::Casualty => false,
            RoundOutcome::NoAction => false,
            _ => true,
        }
    }

    fn is_stable(self) -> bool {
        match self {
            RoundOutcome::CombatOnly => true,
            _ => false,
        }
    }
}

#[derive(Debug, Fail)]
pub enum RoundError {
    #[fail(display = "No moves remain on the map.")]
    NoMovesRemain,

    #[fail(display = "Game interrupted: {}", _0)]
    Interrupted(Box<Error>),
}

#[derive(Debug, Clone)]
pub struct Map {
    pub grid: Grid,
    pub sprites: Sprites,
    round: BinaryHeap<Point>,
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

impl Map {
    pub fn new() -> Self {
        Self {
            grid: Grid::new(),
            sprites: Sprites::new(),
            round: BinaryHeap::new(),
        }
    }

    pub fn status(&self) -> MapStatusView {
        MapStatusView { map: self }
    }

    pub fn bbox(&self) -> BoundingBox {
        self.grid.bbox().union(&self.sprites.bbox())
    }

    pub fn element(&self, position: Point) -> MapElement {
        if let Some(sprite) = self.sprites.get(position) {
            MapElement::Sprite(sprite.species())
        } else {
            MapElement::Tile(self.grid.get(position))
        }
    }

    fn victorious(&self) -> Option<Species> {
        self.sprites.peek().and_then(|s| {
            let sp = s.species();
            if !self.sprites.iter().any(|s| s.species().is_enemy(sp)) {
                Some(sp)
            } else {
                None
            }
        })
    }

    /// What point should this sprite be targeting?
    fn target(&self, sprite: &Sprite) -> Option<Point> {
        let mut targets = Vec::new();
        for target in self.sprites.iter().filter(|s| sprite.is_enemy(s)) {
            if sprite.in_range(target.position()) {
                targets.push(target);
            }
        }
        targets.sort_by(|a, b| {
            a.health()
                .cmp(&b.health())
                .then(a.position().cmp(&b.position()).reverse())
        });
        targets.get(0).map(|s| s.position())
    }

    /// The set of possible target points which are in range
    /// of an enemy.
    fn target_points(&self, species: Species) -> HashSet<Point> {
        let mut targets = HashSet::new();
        for sprite in self
            .sprites
            .iter()
            .filter(|s| species.is_enemy(s.species()))
        {
            for position in &sprite.position().adjacent() {
                if self.element(*position).is_empty() {
                    targets.insert(*position);
                }
            }
        }
        targets
    }

    fn find_path(&self, sprite: &Sprite) -> Option<SpritePath> {
        let targets = self.target_points(sprite.species());

        let mut visited = HashSet::new();

        let mut paths = VecDeque::from_iter(
            SpritePath::paths(sprite.position())
                .into_iter()
                .filter(|p| self.element(p.destination).is_empty()),
        );

        while !paths.is_empty() {
            let candidate = paths.pop_front().unwrap();

            if targets.contains(&candidate.destination) {
                return Some(candidate);
            }

            visited.insert(candidate.destination);
            for direction in Direction::all() {
                let next_point = candidate.destination.step(direction);
                if !visited.contains(&next_point) && self.element(next_point).is_empty() {
                    paths.push_back(candidate.extend(direction))
                }
            }
        }

        None
    }

    fn init_round(&mut self) {
        self.round = self.sprites.positions().cloned().collect();
    }

    pub fn round(&mut self, outcome: RoundOutcome) -> RoundOutcome {
        self.init_round();
        let mut outcome = outcome;

        while (!self.round.is_empty()) && (!outcome.is_finished()) {
            outcome = self.tick(outcome);
        }

        // Check if we hit a end-round victory
        if let Some(victor) = self.victorious() {
            if !outcome.is_finished() {
                return RoundOutcome::Victory(victor);
            }
        }

        outcome
    }

    fn direction(&mut self, location: Point, outcome: RoundOutcome) -> Option<Direction> {
        self.sprites.get(location).and_then(|sprite| {
            if self.target(sprite).is_none() {
                self.find_path(sprite).map(|path| path.direction)
            } else {
                None
            }
        })
    }

    pub fn tick(&mut self, outcome: RoundOutcome) -> RoundOutcome {
        let mut outcome = outcome;
        if let Some(victor) = self.victorious() {
            return RoundOutcome::MidRoundVictory(victor);
        }

        if let Some(location) = self.round.pop() {
            // First, the pathfinding phase
            let direction = self.direction(location, outcome);

            // Now movement
            let location = if let Some(d) = direction {
                self.sprites.step(location, d);
                outcome = outcome.movement();
                location.step(d)
            } else {
                location
            };

            // Next, the attack phase
            if let Some(target) = self.sprites.get(location).and_then(|s| self.target(s)) {
                outcome = match self.sprites.attack(location, target) {
                    SpriteStatus::Alive(_) => outcome.combat(),
                    SpriteStatus::Dead => outcome.casualty(),
                };
            }
        }

        outcome
    }

    pub fn score(&self) -> Health {
        self.sprites.iter().map(|s| s.health()).sum()
    }

    pub fn run<F>(&mut self, mut f: F) -> Result<RunOutcome, RoundError>
    where
        F: FnMut(&Self, u32) -> Result<(), Box<Error>>,
    {
        let mut outcome = RoundOutcome::NoAction;
        for round in 1.. {
            f(&self, round).map_err(RoundError::Interrupted)?;
            match self.round(outcome) {
                RoundOutcome::Victory(s) => {
                    return Ok(RunOutcome {
                        rounds: round,
                        victors: s,
                        score: round * self.score(),
                    })
                }
                RoundOutcome::MidRoundVictory(s) => {
                    return Ok(RunOutcome {
                        rounds: round - 1,
                        victors: s,
                        score: (round - 1) * self.score(),
                    })
                }
                RoundOutcome::NoAction => return Err(RoundError::NoMovesRemain),
                o => {
                    outcome = o;
                }
            }
        }
        unreachable!()
    }
}

#[derive(Debug)]
pub struct RunOutcome {
    pub victors: Species,
    pub rounds: u32,
    pub score: Health,
}

impl fmt::Display for RunOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} win after {} rounds for a total score of {}",
            self.victors.plural(),
            self.rounds,
            self.score
        )
    }
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bbox = self.bbox().margin(1);
        for y in bbox.vertical() {
            for x in bbox.horizontal() {
                let point = Point::new(x, y);

                if let Some(sprite) = self.sprites.get(point) {
                    write!(f, "{}", sprite.glyph())?;
                } else {
                    write!(f, "{}", self.grid.get(point))?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct MapStatusView<'m> {
    map: &'m Map,
}

impl<'m> fmt::Display for MapStatusView<'m> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bbox = self.map.bbox().margin(1);
        let mut sprites = Vec::new();
        for y in bbox.vertical() {
            sprites.clear();
            for x in bbox.horizontal() {
                let point = Point::new(x, y);

                if let Some(sprite) = self.map.sprites.get(point) {
                    write!(f, "{}", sprite.glyph())?;
                    sprites.push(sprite.info());
                } else {
                    write!(f, "{}", self.map.grid.get(point))?;
                }
            }

            let info: Vec<String> = sprites.iter().map(|s| s.to_string()).collect();

            write!(f, "   {}", info.join(", "))?;
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Fail)]
pub enum ParseMapError {
    #[fail(display = "Invalid Sprite: {}", _0)]
    InvalidSprite(ParseSpeciesError),

    #[fail(display = "Invalid Tile: {}", _0)]
    InvalidTile(ParseTileError),

    #[fail(display = "Invalid Position: {}", _0)]
    InvalidPosition(TryFromIntError),
}

impl From<TryFromIntError> for ParseMapError {
    fn from(error: TryFromIntError) -> Self {
        ParseMapError::InvalidPosition(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapElement {
    Tile(Tile),
    Sprite(Species),
}

impl MapElement {
    pub fn is_empty(self) -> bool {
        match self {
            MapElement::Tile(Tile::Empty) => true,
            MapElement::Tile(Tile::Wall) => false,
            MapElement::Sprite(_) => false,
        }
    }
}

impl FromStr for MapElement {
    type Err = ParseMapError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(tile) = s.parse::<Tile>() {
            return Ok(MapElement::Tile(tile));
        }
        s.parse::<Species>()
            .map(MapElement::Sprite)
            .map_err(ParseMapError::InvalidSprite)
    }
}

#[derive(Debug, Clone)]
pub struct MapBuilder {
    sprite: SpriteBuilder,
}

impl Default for MapBuilder {
    fn default() -> Self {
        Self {
            sprite: SpriteBuilder::default(),
        }
    }
}

impl MapBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(&self, s: &str) -> Result<Map, ParseMapError> {
        let mut map = Map::new();

        for (y, line) in s.lines().enumerate() {
            if !line.trim().is_empty() {
                for (x, c) in line.trim().chars().enumerate() {
                    let point = Point::new(isize::try_from(x)?, isize::try_from(y)?);
                    match c.to_string().parse::<MapElement>()? {
                        MapElement::Tile(tile) => {
                            map.grid.insert(point, tile);
                        }
                        MapElement::Sprite(species) => {
                            map.sprites.place(self.sprite.build(point, species));
                            map.grid.insert(point, Tile::Empty);
                        }
                    }
                }
            }
        }

        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::examples::{map_ascii_trim, CombatExample};
    use std::fs;
    use std::io::Read;
    use std::path::PathBuf;

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

    #[test]
    fn sprites() {
        let mut s = Sprites::new();

        let sprite = Sprite::new(Species::Elf, Point::new(1, 1), 200, 3);
        s.place(sprite.clone());

        let other = Sprite::new(Species::Elf, Point::new(1, 3), 200, 3);
        s.place(other.clone());

        let mut spos = s.positions().cloned().collect::<Vec<_>>();
        spos.sort();

        assert_eq!(spos, vec![Point::new(1, 3), Point::new(1, 1)]);
    }

    macro_rules! example_map {
        ($n:expr) => {
            include_str!(concat!("../examples/", $n, ".txt"))
        };
    }

    fn trim(s: &str) -> String {
        map_ascii_trim(s)
    }

    #[test]
    fn map() {
        let builder = MapBuilder::default();
        let raw_map = example_map!("simple");
        let example_map = builder.build(raw_map).unwrap();

        assert_eq!(trim(raw_map), trim(&example_map.to_string()));
        assert_eq!(example_map.sprites.len(), 7);
    }

    #[test]
    fn pathfinding_simple() {
        let builder = MapBuilder::default();
        let raw_map = example_map!("pathfinding_simple");
        let example_map = builder.build(raw_map).unwrap();

        assert_eq!(trim(raw_map), trim(&example_map.to_string()));
        assert_eq!(example_map.sprites.len(), 4);

        let sprite = example_map.sprites.peek().unwrap();
        let path = example_map.find_path(sprite);
        assert_eq!(
            path,
            Some(SpritePath {
                destination: Point::new(3, 1),
                direction: Direction::Right,
                distance: 2,
            })
        );
    }

    #[test]
    fn pathfinding_multi() {
        let builder = MapBuilder::default();
        let raw_map = example_map!("pathfinding_multi");
        let example_map = builder.build(raw_map).unwrap();

        assert_eq!(trim(raw_map), trim(&example_map.to_string()));
        assert_eq!(example_map.sprites.len(), 2);

        let sprite = example_map.sprites.peek().unwrap();
        let path = example_map.find_path(sprite);
        assert_eq!(
            path,
            Some(SpritePath {
                destination: Point::new(4, 2),
                direction: Direction::Right,
                distance: 3,
            })
        );
    }

    #[test]
    fn movement() {
        let builder = MapBuilder::default();
        let mut example_map = builder.build(example_map!("movement/1")).unwrap();

        let maps = vec![
            example_map!("movement/1"),
            example_map!("movement/2"),
            example_map!("movement/3"),
            example_map!("movement/4"),
        ];

        assert_eq!(
            example_map.direction(Point::new(1, 1), RoundOutcome::NoAction),
            Some(Direction::Right)
        );

        // assert_eq!(
        //     example_map.direction(Point::new(1, 1), RoundOutcome::CombatOnly),
        //     None
        // );

        assert_eq!(
            example_map.direction(Point::new(1, 1), RoundOutcome::Casualty),
            Some(Direction::Right)
        );

        assert_eq!(trim(maps[0]), trim(&example_map.to_string()));

        {
            let mut em = example_map.clone();
            em.init_round();
            assert_eq!(em.tick(RoundOutcome::NoAction), RoundOutcome::Movement);
        }

        for (i, raw_map) in maps.iter().enumerate() {
            assert_eq!(
                trim(&example_map.to_string()),
                trim(raw_map),
                "Map doesn't line up at {}\ngot:\n{}\nexpected:\n{}",
                i,
                trim(&example_map.to_string()),
                trim(raw_map)
            );
            example_map.round(RoundOutcome::NoAction);
        }

        assert_eq!(trim(maps[3]), trim(&example_map.to_string()));
    }

    fn load_combat_round(round: usize) -> Option<String> {
        let path = PathBuf::from(format!("examples/combat/{}.txt", round));

        if path.exists() {
            let mut buffer = String::new();
            fs::File::open(path)
                .unwrap()
                .read_to_string(&mut buffer)
                .unwrap();
            Some(buffer)
        } else {
            None
        }
    }

    #[test]
    fn combat() {
        let builder = MapBuilder::default();
        let mut example_map = builder.build(example_map!("combat/initial")).unwrap();

        if let Some(raw_map) = load_combat_round(1) {
            assert_eq!(
                trim(&raw_map),
                trim(&example_map.status().to_string()),
                "combat mismatch on round {}",
                1
            )
        }

        let mut outcome = RoundOutcome::NoAction;
        for round in 2..50 {
            outcome = example_map.round(outcome);

            if let Some(raw_map) = load_combat_round(round) {
                assert_eq!(
                    trim(&raw_map),
                    trim(&example_map.status().to_string()),
                    "combat mismatch on round {}",
                    round
                )
            }
        }

        assert_eq!(example_map.score() * 47, 27730);
    }

    macro_rules! check_example {
        ($e:expr) => {
            let r = $e.check();
            assert!(
                r.is_ok(),
                "Error: {:?}",
                r.map_err(|e| {
                    eprintln!("{}", e);
                    e.to_string()
                })
            );

            assert_eq!($e.check().unwrap(), ());
        };
    }

    #[test]
    fn examples_part1() {
        check_example!(CombatExample::from_str(example_map!("example1")).unwrap());
        check_example!(CombatExample::from_str(example_map!("example2")).unwrap());
        check_example!(CombatExample::from_str(example_map!("example3")).unwrap());
        check_example!(CombatExample::from_str(example_map!("example4")).unwrap());
        check_example!(CombatExample::from_str(example_map!("example5")).unwrap());
    }

    #[test]
    fn examples_part1_example1() {
        let ce = CombatExample::from_str(example_map!("example1")).unwrap();
        let mut map = ce.map.clone();

        assert_eq!(map.round(RoundOutcome::NoAction), RoundOutcome::Movement);
    }

}
