#![allow(dead_code)]

use std::num::ParseIntError;
use std::str::FromStr;

use failure::Error;
use regex::Regex;

use crate::game::Game;
use crate::map::{Map, MapBuilder, ParseMapError};
use crate::sprite::{Health, Species};

#[derive(Debug)]
pub struct CombatExample {
    pub map: Map,
    outcome: String,
    rounds: u32,
    score: Health,
    health: Health,
    victor: Species,
}

impl CombatExample {
    pub(crate) fn check(&self) -> Result<(), Error> {
        let mut game = Game::new(self.map.clone());
        let outcome = game.run(|_, _| Ok(()))?;

        if map_ascii_trim(&game.map().status().to_string()) != map_ascii_trim(&self.outcome) {
            return Err(format_err!(
                "Outcome map doesn't match:\nGot:\n{}\nExpected:\n{}",
                map_ascii_trim(&game.map().status().to_string()),
                map_ascii_trim(&self.outcome)
            ));
        };

        if self.rounds != outcome.rounds {
            return Err(format_err!(
                "Rounds don't match:\nGot: {} Expected: {}",
                outcome.rounds,
                self.rounds
            ));
        }

        if self.victor != outcome.victors {
            return Err(format_err!(
                "Victor doesn't match:\nGot: {} Expected: {}",
                outcome.victors,
                self.victor
            ));
        }

        if self.score != outcome.score {
            return Err(format_err!(
                "Score don't match:\nGot: {} Expected: {}",
                outcome.score,
                self.score
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Fail)]
pub enum CombatExampleParseError {
    #[fail(display = "Invalid Map: {}", _0)]
    InvalidMap(ParseMapError),

    #[fail(display = "Invalid number: {}", _0)]
    InvalidNumber(ParseIntError),

    #[fail(display = "Invalid victor: {}", _0)]
    InvalidVictor(String),

    #[fail(display = "Missing Part: {}", _0)]
    MissingPart(String),

    #[fail(display = "Invalid Meta Line: {}", _0)]
    InvalidMeta(String),
}

impl From<ParseIntError> for CombatExampleParseError {
    fn from(error: ParseIntError) -> Self {
        CombatExampleParseError::InvalidNumber(error)
    }
}

impl FromStr for CombatExample {
    type Err = CombatExampleParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut map = Vec::new();
        let mut outcome = Vec::new();

        let mut lines = s.lines();

        for line in lines.by_ref() {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"(#\S+)\s*(-->)?\s*(#.+)").unwrap();
            };

            let cap = match RE.captures(line) {
                None => {
                    break;
                }
                Some(cap) => cap,
            };

            map.push(cap[1].to_string());
            outcome.push(cap[3].to_string());
        }

        let map = map.join("\n");
        let outcome = outcome.join("\n");

        let mut rounds = None;
        let mut score = None;
        let mut health = None;
        let mut victor = None;

        for line in lines {
            lazy_static! {
                static ref REROUNDS: Regex =
                    Regex::new(r"Combat ends after (\d+) full rounds").unwrap();
            };

            match REROUNDS.captures(line) {
                None => {}
                Some(cap) => {
                    rounds = Some(cap[1].parse::<Health>()?);
                    continue;
                }
            };

            lazy_static! {
                static ref REVICTORY: Regex =
                    Regex::new(r"(\w+) win with (\d+) total hit points left").unwrap();
            };

            match REVICTORY.captures(line) {
                None => {}
                Some(cap) => {
                    health = Some(cap[2].parse::<Health>()?);
                    victor = Some(match &cap[1] {
                        "Elves" => Species::Elf,
                        "Goblins" => Species::Goblin,
                        s => return Err(CombatExampleParseError::InvalidVictor(s.to_string())),
                    });
                    continue;
                }
            };

            lazy_static! {
                static ref REOUTCOME: Regex =
                    Regex::new(r"Outcome: ([\d]+) \* ([\d]+) = ([\d]+)").unwrap();
            };

            match REOUTCOME.captures(line) {
                None => {}
                Some(cap) => {
                    rounds = Some(cap[1].parse::<Health>()?);
                    health = Some(cap[2].parse::<Health>()?);
                    score = Some(cap[3].parse::<Health>()?);
                    continue;
                }
            };

            return Err(CombatExampleParseError::InvalidMeta(line.to_string()));
        }

        let rounds =
            rounds.ok_or_else(|| CombatExampleParseError::MissingPart("rounds".to_owned()))?;
        let health =
            health.ok_or_else(|| CombatExampleParseError::MissingPart("health".to_owned()))?;
        let score =
            score.ok_or_else(|| CombatExampleParseError::MissingPart("score".to_owned()))?;

        let victor =
            victor.ok_or_else(|| CombatExampleParseError::MissingPart("victor".to_owned()))?;

        let map: Map = MapBuilder::default()
            .build(&map)
            .map_err(CombatExampleParseError::InvalidMap)?;

        Ok(CombatExample {
            map,
            outcome,
            rounds,
            health,
            score,
            victor,
        })
    }
}

pub(crate) fn map_ascii_trim(s: &str) -> String {
    let parts = s
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<&str>>();
    parts.join("\n")
}
