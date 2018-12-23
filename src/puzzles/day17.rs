use failure::{format_err, Error};

use geometry::Point;
use waterfall::{Ground, Scan, Water, WellSystem};

pub(crate) fn main() -> Result<(), Error> {
    use crate::input_to_string;

    let scans = input_to_string(17)?
        .lines()
        .map(|l| l.parse::<Scan>())
        .collect::<Result<Vec<_>, _>>()?;

    let ground = Ground::from_scans(Point::new(500, 0), &scans);
    let mut system = WellSystem::new(ground, Water::new());

    system
        .fill()
        .last()
        .ok_or_else(|| format_err!("No water flowed"))?;
    println!("Part 1: {}", system.wet());
    println!("Part 2: {}", system.retained());

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn answer() {
        use crate::input_to_string;

        let scans = input_to_string(17)
            .unwrap()
            .lines()
            .map(|l| l.parse::<Scan>())
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let ground = Ground::from_scans(Point::new(500, 0), &scans);
        let mut system = WellSystem::new(ground, Water::new());

        system
            .fill()
            .last()
            .ok_or_else(|| format_err!("No water flowed"))
            .unwrap();

        assert_eq!(system.wet(), 31412);
        assert_eq!(system.retained(), 25857);
    }
}
