use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::Duration;

use failure::{format_err, Error};

use structopt::StructOpt;

use cursive::traits::*;
use cursive::view::Selector;
use cursive::Cursive;

use geometry::Point;
use waterfall::views::layout;
use waterfall::{Ground, Scan, Water, WellSystem};

#[derive(Debug, StructOpt)]
#[structopt(name = "water", about = "Show the results of a water scan.")]
struct Opt {
    /// Parse as a map
    #[structopt(short = "m", long = "map")]
    map: bool,

    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    let system = if opt.map {
        read_drawn_system(opt.input)?
    } else {
        read_scanned_system(opt.input)?
    };

    let mut siv = setup()?;
    siv.set_fps(10);

    let wait = Duration::from_millis(100);
    siv.add_layer(layout(system, wait).full_screen());
    siv.focus(&Selector::Id("layout")).unwrap();
    siv.run();

    Ok(())
}

fn read_drawn_system<P>(p: P) -> Result<WellSystem, Error>
where
    P: AsRef<Path>,
{
    let mut b = String::new();
    fs::File::open(p)?.read_to_string(&mut b)?;

    let ground: Ground = b.parse()?;
    Ok(WellSystem::new(ground, Water::new()))
}

fn read_scanned_system<P>(p: P) -> Result<WellSystem, Error>
where
    P: AsRef<Path>,
{
    let scans: Vec<Scan> = BufReader::new(fs::File::open(p)?)
        .lines()
        .map(|l| {
            l.map_err(Error::from)
                .and_then(|l| l.parse::<Scan>().map_err(Error::from))
        })
        .collect::<Result<Vec<Scan>, _>>()?;

    let ground = Ground::from_scans(Point::new(500, 0), &scans);
    Ok(WellSystem::new(ground, Water::new()))
}

fn setup() -> Result<Cursive, Error> {
    let mut siv = Cursive::default();

    siv.load_toml(include_str!("theme.toml"))
        .map_err(|e| format_err!("{:?}", e))?;

    siv.add_global_callback('q', |s| s.quit());
    Ok(siv)
}
