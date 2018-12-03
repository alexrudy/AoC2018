#[macro_use]
extern crate serde_derive;
extern crate docopt;

use docopt::Docopt;
use std::fs::File;
use std::io::BufReader;

mod puzzles;

const USAGE: &str = "
Advent of Code 2018.

Usage:
    aoc2018 <day>

";

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

fn main() -> Result {
    let args: Args = Docopt::new(USAGE)
        .and_then(|dopt| dopt.deserialize())
        .unwrap_or_else(|e| e.exit());

    println!("Solving AoC for Day {}", args.arg_day);

    let solvers: Vec<Box<Fn() -> Result>> =
        vec![Box::new(puzzles::day1::main), Box::new(puzzles::day2::main)];

    if args.arg_day < solvers.len() {
        eprintln!("Can't solve puzzle for day {}", args.arg_day);
        Ok(())
    } else {
        (solvers[args.arg_day - 1])()
    }
}
