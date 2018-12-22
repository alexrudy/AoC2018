#![macro_use]

#[macro_use]
extern crate serde_derive;
extern crate docopt;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate itertools;

extern crate rayon;

extern crate regex;

extern crate carts;

#[macro_use]
extern crate failure;

extern crate goblinwars;

use docopt::Docopt;

use std::fs::File;
use std::io::BufReader;

#[macro_export]
macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<::std::error::Error>::from(format!($($tt)*))) }
}

#[macro_export]
macro_rules! newerr {
    ($($tt:tt)*) => { Box::<::std::error::Error>::from(format!($($tt)*)) }
}

mod puzzles;

const USAGE: &str = "
Advent of Code 2018.

Usage:
    aoc2018 <day>

";

macro_rules! day {
    ($iden:ident) => {
        Box::new(puzzles::$iden::main)
    };
}

#[derive(Deserialize)]
struct Args {
    arg_day: usize,
}

type Result = ::std::result::Result<(), Box<::std::error::Error>>;

pub fn input(day: usize) -> std::io::Result<Box<::std::io::BufRead>> {
    let mut p = ::std::path::PathBuf::from("puzzles");
    p.push(format!("{}", day));
    p.push("input.txt");

    let f = File::open(p)?;

    Ok(Box::new(BufReader::new(f)))
}
pub fn input_to_string(day: usize) -> std::io::Result<String> {
    let mut buffer = String::new();
    input(day)?.read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn main() -> Result {
    let args: Args = Docopt::new(USAGE)
        .and_then(|dopt| dopt.deserialize())
        .unwrap_or_else(|e| e.exit());

    println!("Solving AoC for Day {}", args.arg_day);

    let solvers: Vec<Box<Fn() -> Result>> = vec![
        day!(day1),
        day!(day2),
        day!(day3),
        day!(day4),
        day!(day5),
        day!(day6),
        day!(day7),
        day!(day8),
        day!(day9),
        day!(day10),
        day!(day11),
        day!(day12),
        day!(day13),
        day!(day14),
        day!(day15),
        day!(day16),
    ];

    if args.arg_day > solvers.len() || args.arg_day < 1 {
        eprintln!("Can't solve puzzle for day {}", args.arg_day);
        Ok(())
    } else {
        (solvers[args.arg_day - 1])()
    }
}
