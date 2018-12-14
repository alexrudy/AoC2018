use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

type Result<T> = ::std::result::Result<T, Box<Error>>;

pub(crate) fn main() -> Result<()> {
    let grid = Grid::new(1133);

    println!(
        "Part 1: {}",
        grid.max_patch()
            .ok_or_else(|| newerr!("No patches found"))?
            .0
    );

    let (coord, size, _) = grid
        .max_vpatch()
        .ok_or_else(|| newerr!("No patches found"))?;

    println!("Part 2: {},{}", coord, size);

    Ok(())
}

type Element = i64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
struct Coordinate {
    x: Element,
    y: Element,
}

impl Coordinate {
    fn new(x: Element, y: Element) -> Self {
        Self { x, y }
    }

    fn power(&self, serial: Element) -> Element {
        let rack_id = self.x + 10;
        let mut power_level = rack_id * self.y;
        power_level += serial;
        power_level *= rack_id;
        power_level = (power_level / 100) % 10;
        power_level -= 5;
        power_level
    }
}

impl FromStr for Coordinate {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return err!("Not the right number of commas: {}", s);
        }

        Ok(Coordinate {
            x: parts[0].parse()?,
            y: parts[1].parse()?,
        })
    }
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
struct Patch {
    origin: Coordinate,
    size: Element,
}

struct Grid {
    serial: Element,
    table: HashMap<Coordinate, Element>,
}

impl Grid {
    fn new(serial: Element) -> Self {
        let mut table = HashMap::new();

        for (x, y) in iproduct!(1..=300, 1..=300) {
            let here = Coordinate::new(x, y);
            let power = here.power(serial);
            let intensity = power
                + *table.get(&Coordinate::new(x, y - 1)).unwrap_or(&0)
                + *table.get(&Coordinate::new(x - 1, y)).unwrap_or(&0)
                - *table.get(&Coordinate::new(x - 1, y - 1)).unwrap_or(&0);
            table.insert(here, intensity);
        }

        Self { serial, table }
    }

    fn power(&self, position: Coordinate) -> Element {
        position.power(self.serial)
    }

    fn vpatch(&self, position: &Coordinate, size: Element) -> Element {
        let tl = Coordinate::new(position.x - 1, position.y - 1);
        let br = Coordinate::new(position.x - 1 + size, position.y - 1 + size);
        let bl = Coordinate::new(position.x - 1, position.y - 1 + size);
        let tr = Coordinate::new(position.x - 1 + size, position.y - 1);

        *self.table.get(&tl).unwrap_or(&0) + *self.table.get(&br).unwrap_or(&0)
            - *self.table.get(&bl).unwrap_or(&0)
            - *self.table.get(&tr).unwrap_or(&0)
    }

    fn patch(&self, position: &Coordinate) -> Element {
        self.vpatch(position, 3)
    }

    fn max_patch(&self) -> Option<(Coordinate, Element)> {
        iproduct!(1..298, 1..298)
            .map(|(x, y)| {
                let c = Coordinate::new(x, y);
                (c, self.patch(&c))
            })
            .max_by_key(|(_, p)| *p)
    }

    fn max_vpatch(&self) -> Option<(Coordinate, Element, Element)> {
        use rayon::prelude::*;

        (1i64..301)
            .into_par_iter()
            .map(|s| {
                iproduct!(1..298, 1..298)
                    .filter(|(x, y)| (x + s <= 300) && (y + s <= 300))
                    .map(|(x, y)| {
                        let c = Coordinate::new(x, y);
                        (c, s, self.vpatch(&c, i64::from(s)))
                    })
                    .max_by_key(|(_, _, p)| *p)
            })
            .max_by_key(|v| v.map(|(_, _, p)| p).unwrap_or(0))
            .unwrap_or(None)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl Patch {
        fn new(origin: Coordinate, size: Element) -> Self {
            Self { origin, size }
        }
    }

    struct PatchDisplay<'g> {
        grid: &'g Grid,
        patch: Patch,
    }

    impl<'g> fmt::Display for PatchDisplay<'g> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            for y in -1..=self.patch.size {
                for x in -1..=self.patch.size {
                    if (x + self.patch.origin.x) < 0 || (y + self.patch.origin.y) < 0 {
                        write!(f, "  _  ")?;
                    } else {
                        write!(
                            f,
                            " {:3} ",
                            self.grid.power(Coordinate::new(
                                x + self.patch.origin.x,
                                y + self.patch.origin.y
                            ))
                        )?;
                    }
                }
                writeln!(f, "")?;
            }
            Ok(())
        }
    }

    impl Grid {
        #[cfg(test)]
        fn view(&self, patch: &Patch) -> PatchDisplay {
            PatchDisplay {
                grid: &self,
                patch: *patch,
            }
        }
    }

    #[test]
    fn example_part1() {
        assert_eq!("3,5".parse::<Coordinate>().unwrap().power(8), 4);
        assert_eq!("122,79".parse::<Coordinate>().unwrap().power(57), -5);
        assert_eq!("217,196".parse::<Coordinate>().unwrap().power(39), 0);
        assert_eq!("101,153".parse::<Coordinate>().unwrap().power(71), 4);

        let grid = Grid::new(18);

        println!(
            "Grid:\n {}",
            grid.view(&Patch::new(Coordinate::new(33, 45), 3))
        );
        println!("Power:\n {}", grid.patch(&Coordinate::new(33, 45)));

        assert_eq!(
            grid.max_patch(),
            Some(("33,45".parse::<Coordinate>().unwrap(), 29))
        );

        let grid = Grid::new(42);
        assert_eq!(
            grid.max_patch(),
            Some(("21,61".parse::<Coordinate>().unwrap(), 30))
        );
    }

    #[test]
    fn example_part2_a() {
        let grid = Grid::new(18);

        assert_eq!(
            grid.max_vpatch(),
            Some(("90,269".parse::<Coordinate>().unwrap(), 16, 113))
        );
    }

    #[test]
    fn example_part2_b() {
        let grid = Grid::new(42);

        assert_eq!(
            grid.max_vpatch(),
            Some(("232,251".parse::<Coordinate>().unwrap(), 12, 119))
        );
    }

}
