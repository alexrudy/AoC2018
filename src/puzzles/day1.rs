use std::collections::HashSet;
use std::io::BufRead;
use std::iter::Iterator;

use failure::Error;

type Result<T> = ::std::result::Result<T, ::std::num::ParseIntError>;

fn parse_frequencies(s: &str) -> Result<i32> {
    s.trim().trim_start_matches('+').parse::<i32>()
}

pub(crate) fn calibrate_frequncy<T>(data: T) -> Result<i32>
where
    T: Iterator<Item = Result<i32>>,
{
    let mut frequency = 0;
    for item in data {
        frequency += item?;
    }
    Ok(frequency)
}

pub(crate) fn repeated_frequency<T>(data: T) -> Result<i32>
where
    T: Iterator<Item = Result<i32>>,
{
    let offsets: Vec<i32> = (data.collect::<Result<Vec<i32>>>())?;
    let mut seen = HashSet::new();

    let mut frequency = 0;
    seen.insert(frequency);

    loop {
        for f in &offsets {
            frequency += f;
            if !seen.insert(frequency) {
                return Ok(frequency);
            }
        }
    }
}

pub(crate) fn part1() -> ::std::result::Result<i32, Error> {
    use crate::input;

    let answer = calibrate_frequncy(input(1)?.lines().map(|l| parse_frequencies(&l.unwrap())))?;
    Ok(answer)
}

pub(crate) fn part2() -> ::std::result::Result<i32, Error> {
    use crate::input;

    let answer = repeated_frequency(input(1)?.lines().map(|l| parse_frequencies(&l.unwrap())))?;
    Ok(answer)
}

pub(crate) fn main() -> ::std::result::Result<(), Error> {
    let frequency = part1()?;
    println!("Part 1: {}", frequency);

    let frequency = part2()?;
    println!("Part 2: {}", frequency);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    fn process<'a>(s: &'a str) -> impl Iterator<Item = Result<i32>> + 'a {
        s.split(',').map(parse_frequencies)
    }

    #[test]
    fn examples_part1() {
        assert_eq!(calibrate_frequncy(process("+1, -2, +3, +1")), Ok(3));
        assert_eq!(calibrate_frequncy(process("+1, +1, +1")), Ok(3));
        assert_eq!(calibrate_frequncy(process("+1, +1, -2")), Ok(0));
        assert_eq!(calibrate_frequncy(process("-1, -2, -3")), Ok(-6));
    }

    #[test]
    fn answer_part1() {
        assert_eq!(part1().unwrap(), 435);
    }

    #[test]
    fn examples_part2() {
        assert_eq!(repeated_frequency(process("+1, -2, +3, +1")), Ok(2));
        assert_eq!(repeated_frequency(process("+1, -1")), Ok(0));
        assert_eq!(repeated_frequency(process("+3, +3, +4, -2, -4")), Ok(10));
        assert_eq!(repeated_frequency(process("-6, +3, +8, +5, -6")), Ok(5));
        assert_eq!(repeated_frequency(process("+7, +7, -2, -7, -4")), Ok(14));
    }

    #[test]
    fn answer_part2() {
        assert_eq!(part2().unwrap(), 245);
    }

}
