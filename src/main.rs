#![macro_use]
#![feature(try_from)]

use failure::Error;

use docopt::Docopt;

use serde_derive::Deserialize;

use std::fs::File;
use std::io::BufReader;

mod elfcode;
mod puzzles;

const USAGE: &str = "
Advent of Code 2018.

Solves a given day's puzzle.

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

fn main() -> Result<(), Error> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|dopt| dopt.deserialize())
        .unwrap_or_else(|e| e.exit());

    println!("Solving AoC for Day {}", args.arg_day);

    let solvers: Vec<Box<Fn() -> Result<(), Error>>> = vec![
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
        day!(day17),
        day!(day18),
        day!(day19),
    ];

    if args.arg_day > solvers.len() || args.arg_day < 1 {
        eprintln!("Can't solve puzzle for day {}", args.arg_day);
        Ok(())
    } else {
        (solvers[args.arg_day - 1])()
    }
}
