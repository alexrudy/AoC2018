use std::collections::{BinaryHeap, HashMap, HashSet};
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io::prelude::*;
use std::path::Path;
use std::str::FromStr;

type ParseResult<T> = Result<T, Box<Error>>;

use crate::cart::{Cart, CartError};
use crate::point::{BBox, Direction, Point};

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<Error>::from(format!($($tt)*))) }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum Track {
    Vertical,
    Horizontal,
    RightCorner,
    LeftCorner,
    Intersection,
    Empty,
}

#[derive(Debug, PartialEq)]
pub enum LayoutError {
    NoCarts,
    OneCart,
    OffTheRails(Point),
    Collision(Point),
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LayoutError::NoCarts => write!(f, "No carts on layout, nothing to run!"),
            LayoutError::OneCart => {
                write!(f, "Can't wait for collision with only one cart on layout!")
            }
            LayoutError::OffTheRails(p) => write!(f, "A cart went off the rails at {}", p),
            LayoutError::Collision(p) => write!(f, "Two carts collided at {}", p),
        }
    }
}

impl Error for LayoutError {}

impl From<CartError> for LayoutError {
    fn from(ce: CartError) -> Self {
        match ce {
            CartError::Collision(p) => LayoutError::Collision(p),
            CartError::OffTheRails(p) => LayoutError::OffTheRails(p),
        }
    }
}

impl From<Direction> for Track {
    fn from(d: Direction) -> Self {
        match d {
            Direction::Up => Track::Vertical,
            Direction::Down => Track::Vertical,
            Direction::Left => Track::Horizontal,
            Direction::Right => Track::Horizontal,
        }
    }
}

impl Default for Track {
    fn default() -> Self {
        Track::Empty
    }
}

impl FromStr for Track {
    type Err = Box<Error>;

    fn from_str(s: &str) -> ParseResult<Self> {
        if s.len() != 1 {
            return err!("Wrong size direction string");
        }

        match s.chars().nth(0).unwrap() {
            '|' => Ok(Track::Vertical),
            '-' => Ok(Track::Horizontal),
            '/' => Ok(Track::RightCorner),
            '\\' => Ok(Track::LeftCorner),
            '+' => Ok(Track::Intersection),
            ' ' => Ok(Track::Empty),
            c => err!("Unknown track: {}", c),
        }
    }
}

impl fmt::Display for Track {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Track::Vertical => write!(f, "|"),
            Track::Horizontal => write!(f, "-"),
            Track::LeftCorner => write!(f, "\\"),
            Track::RightCorner => write!(f, "/"),
            Track::Intersection => write!(f, "+"),
            Track::Empty => write!(f, " "),
        }
    }
}

#[derive(Debug)]
pub struct Layout {
    track: HashMap<Point, Track>,
    carts: BinaryHeap<Cart>,
}

impl Layout {
    fn new() -> Self {
        Self {
            track: HashMap::new(),
            carts: BinaryHeap::new(),
        }
    }

    fn bbox(&self) -> BBox {
        let mut bbox = BBox::empty();
        for point in self.track.keys() {
            bbox.include(*point);
        }
        bbox
    }

    fn lay_track(&mut self, position: Point, track: Track) {
        match track {
            Track::Empty => None,
            t => self.track.insert(position, t),
        };
    }

    fn cart_mapping(&self) -> HashMap<Point, Direction> {
        self.carts
            .iter()
            .map(|c| (c.position, c.direction))
            .collect()
    }

    fn tick(&mut self) -> Result<(), CartError> {
        let mut carts = Vec::with_capacity(self.carts.len());

        let mut positions: HashSet<Point> = self.carts.iter().map(|c| c.position).collect();

        let mut result = Ok(());

        while !self.carts.is_empty() {
            let mut cart = self.carts.pop().unwrap();

            let track = *self.track.get(&cart.position).unwrap_or(&Track::Empty);

            // Don't proceed if this cart actually isn't on the track.
            if positions.remove(&cart.position) {
                // Move the cart forward
                cart.advance(track)
                    .map_err(|e| CartError::from_advance(e, cart.position))?;

                // If this is a collision, remove it and set the result. Otherwise,
                // push the cart back onto the stack.
                if !positions.insert(cart.position) {
                    positions.remove(&cart.position);
                    result = Err(CartError::Collision(cart.position));
                } else {
                    carts.push(cart);
                }
            }
        }

        // Remove any remaining carts
        self.carts = carts
            .into_iter()
            .filter(|c| positions.contains(&c.position))
            .collect();

        result
    }

    pub fn run_until_collision<F>(&mut self, f: F) -> Result<Point, LayoutError>
    where
        F: Fn(&Self),
    {
        if self.carts.is_empty() {
            return Err(LayoutError::NoCarts);
        }

        if self.carts.len() < 2 {
            return Err(LayoutError::OneCart);
        }

        loop {
            match self.tick() {
                Ok(()) => f(&self),
                Err(CartError::Collision(point)) => return Ok(point),
                Err(e) => return Err(e.into()),
            }
        }
    }

    pub fn run_until_last_cart<F>(&mut self, f: F) -> Result<Point, LayoutError>
    where
        F: Fn(&Self),
    {
        if self.carts.is_empty() {
            return Err(LayoutError::NoCarts);
        }

        if self.carts.len() < 2 {
            return Err(LayoutError::OneCart);
        }

        loop {
            match self.tick() {
                Ok(()) => f(&self),
                Err(CartError::Collision(_)) => {}
                Err(e) => return Err(e.into()),
            }
            if self.carts.len() == 1 {
                return Ok(self.carts.peek().unwrap().position);
            }
            if self.carts.is_empty() {
                return Err(LayoutError::NoCarts);
            }
        }
    }

    pub fn from_file<P>(path: P) -> Result<Self, Box<Error>>
    where
        P: AsRef<Path>,
    {
        let mut file = fs::File::open(path)?;
        let data = {
            let mut buffer = String::new();
            file.read_to_string(&mut buffer)?;
            buffer
        };
        Ok(data.parse()?)
    }
}

enum Element {
    Track(Track),
    Cart(Direction),
}

impl FromStr for Element {
    type Err = Box<Error>;

    fn from_str(s: &str) -> ParseResult<Self> {
        match s.parse::<Track>() {
            Ok(t) => Ok(Element::Track(t)),
            Err(_) => Ok(Element::Cart(
                s.parse::<Direction>()
                    .or_else(|_| err!("Unknown track or cart: {}", s))?,
            )),
        }
    }
}

impl FromStr for Layout {
    type Err = Box<Error>;

    fn from_str(s: &str) -> ParseResult<Self> {
        let mut layout = Self::new();

        for (y, line) in s.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                let point = Point::new(i32::try_from(x)?, i32::try_from(y)?);
                match c.to_string().parse::<Element>()? {
                    Element::Track(t) => {
                        layout.lay_track(point, t);
                    }
                    Element::Cart(direction) => {
                        layout.lay_track(point, direction.into());
                        let cart = Cart::new(point, direction);
                        layout.carts.push(cart);
                    }
                }
            }
        }

        Ok(layout)
    }
}

impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bbox = self.bbox();

        let carts = self.cart_mapping();

        for y in 0..bbox.height() {
            for x in 0..bbox.width() {
                let point = Point::new(x, y);

                if let Some(direction) = carts.get(&point) {
                    write!(f, "{}", direction)?;
                } else if let Some(track) = self.track.get(&point) {
                    write!(f, "{}", track)?;
                } else {
                    write!(f, " ")?;
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

    fn trim_lines(s: &str) -> String {
        s.lines().map(|l| format!("{}\n", l.trim())).collect()
    }

    macro_rules! example_layout_file {
        ($n:expr) => {
            include_str!(concat!("../layouts/", $n, ".txt"))
        };
    }

    macro_rules! compare_layout {
        ($n:expr) => {
            assert_eq!(
                trim_lines(&example_layout_file!($n)),
                trim_lines(
                    &Layout::from_str(example_layout_file!($n))
                        .unwrap()
                        .to_string()
                )
            )
        };
    }

    #[test]
    fn example_layouts() {
        compare_layout!("one_loop");
        compare_layout!("two_loop");
        compare_layout!("part1_example");
    }

    #[test]
    fn example_part1() {
        let mut layout = Layout::from_str(example_layout_file!("part1_example")).unwrap();
        let collision = layout.run_until_collision(|_| {}).unwrap();
        assert_eq!(collision, Point::new(7, 3))
    }

    #[test]
    fn example_part2() {
        let mut layout = Layout::from_str(example_layout_file!("part2_example")).unwrap();
        let last_cart = layout.run_until_last_cart(|_| {}).unwrap();
        assert_eq!(last_cart, Point::new(6, 4))
    }

    #[test]
    fn collision_removes_both() {
        let mut layout = Layout::from_str(example_layout_file!("test_collision")).unwrap();
        assert_eq!(layout.tick(), Err(CartError::Collision(Point::new(3, 0))));

        assert_eq!(layout.carts.len(), 1);
    }
}