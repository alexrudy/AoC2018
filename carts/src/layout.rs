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
pub enum Track {
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
    OneCart(Point),
    OffTheRails(Point),
    Collision(Point),
    LastCollision(Point, Point),
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LayoutError::NoCarts => write!(f, "No carts on layout, nothing to run!"),
            LayoutError::OneCart(p) => write!(f, "Only one cart remains, at {}!", p),
            LayoutError::OffTheRails(p) => write!(f, "A cart went off the rails at {}", p),
            LayoutError::Collision(p) => write!(f, "Two carts collided at {}", p),
            LayoutError::LastCollision(pcrash, pcart) => write!(
                f,
                "Two carts collided at {}, last cart is at {}",
                pcrash, pcart
            ),
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

#[derive(Debug, Clone)]
pub struct Layout {
    track: HashMap<Point, Track>,
    carts: BinaryHeap<Cart>,
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}

impl Layout {
    pub fn new() -> Self {
        Self {
            track: HashMap::new(),
            carts: BinaryHeap::new(),
        }
    }

    pub fn bbox(&self) -> BBox {
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

    pub fn cart_mapping(&self) -> HashMap<Point, Direction> {
        self.carts
            .iter()
            .map(|c| (c.position, c.direction))
            .collect()
    }

    pub fn get_track(&self, position: &Point) -> Option<&Track> {
        self.track.get(position)
    }

    fn tick(&mut self) -> Result<(), LayoutError> {
        if self.carts.is_empty() {
            return Err(LayoutError::NoCarts);
        }

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
                    .map_err(|e| LayoutError::from(CartError::from_advance(e, cart.position)))?;

                // If this is a collision, remove it and set the result. Otherwise,
                // push the cart back onto the stack.
                if !positions.insert(cart.position) {
                    positions.remove(&cart.position);
                    if result.is_ok() {
                        result = Err(LayoutError::Collision(cart.position));
                    }
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

        if self.carts.len() == 1 {
            return match result {
                Ok(_) => Err(LayoutError::OneCart(self.carts.peek().unwrap().position)),
                Err(LayoutError::Collision(p)) => Err(LayoutError::LastCollision(
                    p,
                    self.carts.peek().unwrap().position,
                )),
                Err(e) => Err(e),
            };
        }

        result
    }

    pub fn run<F>(&mut self, mut f: F, until: LayoutComplete) -> Result<(), LayoutError>
    where
        F: FnMut(&Self),
    {
        for counter in 0.. {
            let result = self.tick();

            if until.is_complete(&result, counter) {
                return result;
            } else {
                f(&self);
            }
        }
        Ok(())
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

#[derive(Debug, Clone, Copy)]
pub enum LayoutComplete {
    Collision,
    LastCart,
    Iterations(usize),
    Never,
}

impl LayoutComplete {
    fn is_complete(&self, tick_result: &Result<(), LayoutError>, counter: usize) -> bool {
        match (self, tick_result) {
            (LayoutComplete::Collision, Err(LayoutError::Collision(_))) => true,
            (LayoutComplete::Collision, Err(LayoutError::LastCollision(_, _))) => true,
            (LayoutComplete::Collision, Err(LayoutError::OneCart(_))) => true,
            (LayoutComplete::LastCart, Err(LayoutError::OneCart(_))) => true,
            (LayoutComplete::LastCart, Err(LayoutError::LastCollision(_, _))) => true,
            (_, Err(LayoutError::NoCarts)) => true,
            (_, Err(LayoutError::OffTheRails(_))) => true,
            (LayoutComplete::Iterations(i), _) => *i <= counter,
            (_, Err(LayoutError::Collision(_))) => false,
            (_, Ok(())) => false,
            (LayoutComplete::Never, _) => false,
        }
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
        let collision = layout.run(|_| {}, LayoutComplete::Collision).unwrap_err();
        assert_eq!(collision, LayoutError::Collision(Point::new(7, 3)))
    }

    #[test]
    fn example_part2() {
        let mut layout = Layout::from_str(example_layout_file!("part2_example")).unwrap();
        let last_cart = match layout.run(|_| {}, LayoutComplete::LastCart).unwrap_err() {
            LayoutError::LastCollision(_, cart) => cart,
            e => panic!("Unexpected error: {}", e),
        };
        assert_eq!(last_cart, Point::new(6, 4));
    }

    #[test]
    fn collision_removes_both() {
        let mut layout = Layout::from_str(example_layout_file!("test_collision")).unwrap();
        assert_eq!(layout.tick(), Err(LayoutError::Collision(Point::new(3, 0))));

        assert_eq!(layout.carts.len(), 1);
    }
}
