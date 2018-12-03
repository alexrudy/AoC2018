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

pub fn input(day: usize) -> std::io::Result<Box<::std::io::BufRead>> {
    let mut p = ::std::path::PathBuf::from("puzzles");
    p.push(format!("{}", day));
    p.push("input.txt");

    let f = File::open(p)?;

    Ok(Box::new(BufReader::new(f)))
}

fn main() -> Result<(), Box<::std::error::Error>> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|dopt| dopt.deserialize())
        .unwrap_or_else(|e| e.exit());

    println!("Solving AoC for Day {}", args.arg_day);

    match args.arg_day {
        1 => puzzles::day1::main(),
        _ => {
            eprintln!("Can't solve puzzle for day {}", args.arg_day);
            Ok(())
        }
    }
}
