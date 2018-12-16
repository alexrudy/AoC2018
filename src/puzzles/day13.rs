use std::error::Error;
use std::io::prelude::*;

type Result<T> = ::std::result::Result<T, Box<Error>>;

use carts::Layout;

pub(crate) fn main() -> Result<()> {
    use crate::input;

    let mut layout: Layout = {
        let mut buffer = String::new();
        input(13)?.read_to_string(&mut buffer)?;
        buffer.parse()?
    };

    let collision = layout.run_until_collision(|_| {})?;
    println!("Part 1: {}", collision);

    let lastcart = layout.run_until_last_cart(|_| {})?;
    println!("Part 2: {}", lastcart);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use carts::Point;

    #[test]
    fn example_part1() {
        let mut layout: Layout = include_str!("../../carts/layouts/part1_example.txt")
            .parse()
            .unwrap();
        let collision = layout.run_until_collision(|_| {}).unwrap();
        assert_eq!(collision, Point::new(7, 3))
    }

    #[test]
    fn example_part2() {
        let mut layout: Layout = include_str!("../../carts/layouts/part2_example.txt")
            .parse()
            .unwrap();
        let last_cart = layout.run_until_last_cart(|_| {}).unwrap();
        assert_eq!(last_cart, Point::new(6, 4))
    }
}
