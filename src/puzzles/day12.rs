use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::io::BufRead;
use std::str::FromStr;

type Result<T> = ::std::result::Result<T, Box<Error>>;

fn get_pots() -> Result<(Pots, Vec<Note>)> {
    use crate::input;

    let mut lines = input(12)?.lines();

    let initial_state = lines
        .by_ref()
        .nth(0)
        .ok_or_else(|| newerr!("No lines to read."))??;
    let pots: Pots = initial_state
        .split(':')
        .nth(1)
        .ok_or_else(|| newerr!("Not enough input on line 1: {}", initial_state))?
        .trim()
        .parse::<Pots>()?;

    let notes = lines
        .skip(1)
        .map(|r| {
            r.map_err(Box::<Error>::from)
                .and_then(|s| s.parse::<Note>())
        })
        .filter(|nr| nr.as_ref().map(|n| n.grow).unwrap_or(true))
        .collect::<Result<Vec<Note>>>()?;
    Ok((pots, notes))
}

fn evolve_until_stable(pots: Pots, notes: &[Note], maxiter: Plant) -> (Pots, Plant) {
    let mut pots = pots;
    let mut offset = 0;
    let mut same = 0;
    {
        let mut seq = pots.sequence();
        for i in 1..maxiter {
            pots = pots.grow_once(&notes);
            println!("{}", pots);

            if pots.sequence() == seq && i > 20 {
                same += 1;
                if same > 2 {
                    offset = i;
                    break;
                }
            } else {
                seq = pots.sequence();
            }
        }
    }
    (pots, offset)
}

pub(crate) fn main() -> Result<()> {
    let (pots, notes) = get_pots()?;

    {
        let mut pots = pots.clone();
        for _ in 0..20 {
            pots = pots.grow_once(&notes);
        }
        println!("Part 1: {}", pots.total());
    }

    {
        let iterations = 50_000_000_000i64;
        let (pots, offset) = evolve_until_stable(pots, &notes, iterations);
        println!("{}", offset);
        println!("Part 2: {}", pots.total_with_offset(iterations - offset));
    }

    Ok(())
}

type Plant = i64;

#[derive(Debug, PartialEq, Clone)]
struct Pots(HashSet<Plant>);

impl Pots {
    fn range(&self) -> (Plant, Plant) {
        let first_plant = *self.0.iter().min().expect("Must have at least one plant!");
        let last_plant = *self.0.iter().max().expect("Must have at least one plant!");
        (first_plant, last_plant)
    }

    fn total(&self) -> Plant {
        self.0.iter().sum()
    }

    fn total_with_offset(&self, base: Plant) -> Plant {
        self.0.iter().map(|p| p + base).sum()
    }

    fn sequence(&self) -> Vec<bool> {
        let (first_plant, last_plant) = self.range();

        (first_plant..=last_plant)
            .map(|i| self.0.contains(&i))
            .collect()
    }

    fn grow_once(&self, notes: &[Note]) -> Pots {
        let mut next_generation = HashSet::new();

        let (first_plant, last_plant) = self.range();

        for i in (first_plant - 2)..=(last_plant + 2) {
            for note in notes {
                if note.grow
                    && (-2..=2).all(|p| self.0.contains(&(i + p)) == note.plants.contains(&p))
                {
                    next_generation.insert(i);
                }
            }
        }

        Pots(next_generation)
    }
}

impl fmt::Display for Pots {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (_, end) = self.range();
        for i in -2..=(end + 2) {
            if self.0.contains(&i) {
                write!(f, "#")?;
            } else {
                write!(f, ".")?;
            }
        }
        Ok(())
    }
}

impl FromStr for Pots {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Pots(
            s.chars()
                .enumerate()
                .filter(|(_, c)| c == &'#')
                .map(|(i, _)| i as Plant)
                .collect::<HashSet<Plant>>(),
        ))
    }
}

#[derive(Debug, PartialEq)]
struct Note {
    plants: HashSet<Plant>,
    grow: bool,
}

impl FromStr for Note {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Self> {
        let mut chars = s.chars();
        let plants: HashSet<Plant> = chars
            .by_ref()
            .take(5)
            .enumerate()
            .filter(|(_, c)| c == &'#')
            .map(|(i, _)| (i as Plant) - 2)
            .collect();

        let arrow: String = chars.by_ref().take(4).collect();
        if &arrow != " => " {
            return err!("Expected to find an arrow: {}", arrow);
        }

        let marker: String = chars.by_ref().collect();
        let grow = match marker.as_str() {
            "." => false,
            "#" => true,
            _ => return err!("Expected a plant marker: {}", marker),
        };

        Ok(Self { plants, grow: grow })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::iter::FromIterator;

    const NOTES: &str = "...## => #
..#.. => #
.#... => #
.#.#. => #
.#.## => #
.##.. => #
.#### => #
#.#.# => #
#.### => #
##.#. => #
##.## => #
###.. => #
###.# => #
####. => #";

    fn construct_notes(s: &str) -> Result<Vec<Note>> {
        s.lines()
            .map(|l| l.parse::<Note>())
            .filter(|nr| nr.as_ref().map(|n| n.grow).unwrap_or(true))
            .collect()
    }

    fn example_notes() -> Vec<Note> {
        construct_notes(NOTES).unwrap()
    }

    #[test]
    fn example_part1() {
        let notes = example_notes();
        let mut pots = "#..#.#..##......###...###".parse::<Pots>().unwrap();

        for _ in 0..10 {
            pots = pots.grow_once(&notes);
        }

        assert_eq!(&format!("{}", pots), ".#.#..#...#.##....##..##..##..##..");

        for _ in 10..20 {
            pots = pots.grow_once(&notes);
        }

        assert_eq!(
            &format!("{}", pots),
            "#....##....#####...#######....#.#..##.."
        );

        assert_eq!(pots.total(), 325);
    }

    #[test]
    fn parse_note() {
        assert_eq!(
            "..#.. => .".parse::<Note>().unwrap(),
            Note {
                plants: vec![0].into_iter().collect(),
                grow: false
            }
        );

        assert_eq!(
            "##.## => .".parse::<Note>().unwrap(),
            Note {
                plants: vec![-2, -1, 1, 2].into_iter().collect(),
                grow: false
            }
        );
    }

    #[test]
    fn parse_pots() {
        assert_eq!(
            "#..##....".parse::<Pots>().unwrap(),
            Pots(HashSet::from_iter(vec![0 as Plant, 3, 4].into_iter()))
        )
    }

    #[test]
    fn answer_part1() {
        let (mut pots, notes) = get_pots().unwrap();

        for _ in 0..20 {
            pots = pots.grow_once(&notes);
        }

        assert_eq!(pots.total(), 4200);
    }

    #[test]
    fn answer_part2() {
        let (pots, notes) = get_pots().unwrap();

        let iterations = 50_000_000_000i64;
        let (pots, offset) = evolve_until_stable(pots, &notes, iterations);
        assert_eq!(
            pots.total_with_offset(iterations - offset),
            9_699_999_999_321
        );
    }

}
