use std::error::Error;
use std::io::prelude::*;

type Result<T> = ::std::result::Result<T, Box<Error>>;

use carts::{Layout, LayoutComplete, LayoutError};

pub(crate) fn main() -> Result<()> {
    use crate::input;

    let mut layout: Layout = {
        let mut buffer = String::new();
        input(13)?.read_to_string(&mut buffer)?;
        buffer.parse()?
    };

    match layout.run(|_| {}, LayoutComplete::Collision) {
        Err(LayoutError::Collision(collision)) => println!("Part 1: {}", collision),
        Err(e) => return Err(e.into()),
        Ok(()) => return err!("Layout ended without a collision!"),
    }

    match layout.run(|_| {}, LayoutComplete::LastCart) {
        Err(LayoutError::OneCart(cart)) => println!("Part 2: {}", cart),
        Err(LayoutError::LastCollision(_, cart)) => println!("Part 2: {}", cart),
        Err(e) => return Err(e.into()),
        Ok(()) => return err!("Layout ended without a collision!"),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use geometry::Point;

    #[test]
    fn example_part1() {
        let mut layout: Layout = include_str!("../../carts/layouts/part1_example.txt")
            .parse()
            .unwrap();
        let collision = layout.run(|_| {}, LayoutComplete::Collision);
        assert_eq!(collision, Err(LayoutError::Collision(Point::new(7, 3))))
    }

    #[test]
    fn example_part2() {
        let mut layout: Layout = include_str!("../../carts/layouts/part2_example.txt")
            .parse()
            .unwrap();
        let last_cart = match layout.run(|_| {}, LayoutComplete::LastCart).unwrap_err() {
            LayoutError::LastCollision(_, cart) => cart,
            e => panic!("Unexpected error: {}", e),
        };
        assert_eq!(last_cart, Point::new(6, 4));
    }
}
