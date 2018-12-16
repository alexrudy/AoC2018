extern crate carts;

#[macro_use]
extern crate serde_derive;

extern crate docopt;

use std::error::Error;
use std::fs;
use std::io::prelude::*;
use std::path::Path;
use std::process::exit;

use std::{thread, time};

use docopt::Docopt;

use carts::Layout;

const USAGE: &str = "
Advent of Code 2018 - Elf Cart Visualizer.

Usage:
    playcart collision <layoutfile>
    playcart last <layoutfile>
";

#[derive(Deserialize)]
struct Args {
    cmd_collision: bool,
    cmd_last: bool,
    arg_layoutfile: String,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        exit(1);
    }
}

fn load_layout<P>(path: P) -> Result<Layout, Box<Error>>
where
    P: AsRef<Path>,
{
    let mut file = fs::File::open(path)?;
    let data = {
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        buffer
    };
    Ok(data.parse()?)
}

fn run() -> Result<(), Box<Error>> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let mut layout = load_layout(args.arg_layoutfile)?;
    print!("{}[2J", 27 as char);
    print!("{}", layout);

    let pause = time::Duration::from_millis(300);

    if args.cmd_collision {
        let collision = layout.run_until_collision(|l| {
            thread::sleep(pause);
            print!("{}[2J", 27 as char);
            print!("{}", l);
        })?;

        print!("{}[2J", 27 as char);
        print!("{}", layout);

        println!("");
        println!("Collision at {}", collision);
    } else if args.cmd_last {
        let collision = layout.run_until_last_cart(|l| {
            thread::sleep(pause);
            print!("{}[2J", 27 as char);
            print!("{}", l);
        })?;

        print!("{}[2J", 27 as char);
        print!("{}", layout);

        println!("");
        println!("Last cart at {}", collision);
    }

    Ok(())
}
