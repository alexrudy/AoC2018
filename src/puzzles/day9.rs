use std::collections::{HashMap, VecDeque};
use std::error::Error;

type Result<T> = ::std::result::Result<T, Box<Error>>;
type Marble = usize;

pub(crate) fn main() -> Result<()> {
    let (players, marbles) = parse_input()?;

    println!("Part 1: {}", play(marbles, players).values().max().unwrap());
    println!(
        "Part 2: {}",
        play(marbles * 100, players).values().max().unwrap()
    );

    Ok(())
}

#[derive(Debug)]
struct Circle {
    marbles: VecDeque<Marble>,
}

impl Circle {
    fn new() -> Self {
        Self {
            marbles: VecDeque::from(vec![0]),
        }
    }

    fn view(&self) -> Vec<Marble> {
        self.marbles.iter().cloned().collect()
    }

    fn add(&mut self, marble: Marble) -> Option<(Marble, Marble)> {
        if marble % 23 == 0 {
            for _ in 0..7 {
                let m = self.marbles.pop_back().unwrap();
                self.marbles.push_front(m);
            }
            let m = self.marbles.pop_back().unwrap();
            {
                let front = self.marbles.pop_front().unwrap();
                self.marbles.push_back(front);
            }

            Some((marble, m))
        } else {
            // Rotate around once.
            let last = self.marbles.pop_front().unwrap();
            self.marbles.push_back(last);
            self.marbles.push_back(marble);
            None
        }
    }
}

fn play(marbles: Marble, players: usize) -> HashMap<usize, Marble> {
    let mut scores = HashMap::new();
    let mut circle = Circle::new();

    for (marble, player) in (1..=marbles).zip((1..(players + 1)).cycle()) {
        if let Some((ma, mb)) = circle.add(marble) {
            *scores.entry(player).or_insert(0) += ma + mb;
        }
    }

    scores
}

fn parse_input() -> Result<(usize, Marble)> {
    use crate::input;
    use regex::Regex;

    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"([\d]+) players; last marble is worth ([\d]+) points").unwrap();
    };

    let data = {
        let mut s = String::new();
        input(9)?.read_to_string(&mut s)?;
        s
    };

    let cap = match RE.captures(&data) {
        Some(c) => c,
        None => return err!("Can't match input: {}", data),
    };

    let players: usize = cap[1].parse()?;
    let marbles: Marble = cap[2].parse()?;

    Ok((players, marbles))
}

#[cfg(test)]
mod test {
    use super::*;

    use regex::Regex;

    #[test]
    fn simple() {
        let mut circle = Circle::new();
        assert_eq!(circle.view(), vec![0]);

        assert_eq!(circle.add(1), None);
        assert_eq!(circle.view(), vec![0, 1]);

        assert_eq!(circle.add(2), None);
        assert_eq!(circle.view(), vec![1, 0, 2]);

        assert_eq!(circle.add(3), None);
        assert_eq!(circle.view(), vec![0, 2, 1, 3]);

        assert_eq!(circle.add(4), None);
        assert_eq!(circle.view(), vec![2, 1, 3, 0, 4]);

        for m in 5..23 {
            assert_eq!(circle.add(m), None);
        }
        assert_eq!(circle.add(23), Some((23, 9)));
        assert_eq!(circle.marbles.back(), Some(&19));
    }

    fn testcase(data: &str) {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"([\d]+) players; last marble is worth ([\d]+) points: high score is ([\d]+)"
            )
            .unwrap();
        }

        let cap = RE.captures(data).unwrap();

        println!("{}/{}/{}", &cap[1], &cap[2], &cap[3]);

        let players: usize = cap[1].parse().unwrap();
        let marbles: Marble = cap[2].parse().unwrap();
        let score: Marble = cap[3].parse().unwrap();

        assert_eq!(play(marbles, players).values().max(), Some(&score));
    }

    #[test]
    fn example_part1() {
        let scores = play(25, 9);
        assert_eq!(scores.len(), 1);
        assert_eq!(scores.get(&5), Some(&32));

        testcase("10 players; last marble is worth 1618 points: high score is 8317");
        testcase("13 players; last marble is worth 7999 points: high score is 146373");
        testcase("17 players; last marble is worth 1104 points: high score is 2764");
        testcase("21 players; last marble is worth 6111 points: high score is 54718");
        testcase("30 players; last marble is worth 5807 points: high score is 37305");
    }

    #[test]
    fn answer_part1() {
        let (players, marbles) = parse_input().unwrap();

        assert_eq!(play(marbles, players).values().max(), Some(&398048))
    }

    #[test]
    fn answer_part2() {
        let (players, marbles) = parse_input().unwrap();

        assert_eq!(
            play(marbles * 100, players).values().max(),
            Some(&3180373421)
        )
    }
}
