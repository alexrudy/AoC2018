extern crate structopt;

#[macro_use]
extern crate failure;

extern crate goblinwars;

use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time;

use failure::Error;

use structopt::StructOpt;

use cursive::view::Boxable;
use cursive::views::{LinearLayout, TextView};
use cursive::Cursive;

use goblinwars::map::{Map, MapBuilder};
use goblinwars::views::{MapView, MessageView};
use goblinwars::CombatExample;

#[derive(Debug, StructOpt)]
#[structopt(name = "wars", about = "Play a goblin wars scenario.")]
struct Opt {
    /// Activate debug mode
    #[structopt(short = "d", long = "debug")]
    debug: bool,

    #[structopt(short = "e", long = "example")]
    example: bool,
    /// Set speed
    #[structopt(short = "s", long = "speed", default_value = "1")]
    speed: u64,
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn load(path: &PathBuf) -> io::Result<String> {
    let mut mbuf = String::new();
    fs::File::open(path)?.read_to_string(&mut mbuf)?;
    Ok(mbuf)
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    let builder = MapBuilder::default();

    let map = if opt.example {
        load(&opt.input)?.parse::<CombatExample>()?.map
    } else {
        builder.build(&load(&opt.input)?)?
    };

    let (tx, rx) = mpsc::channel();
    let (tx_message, rx_message) = mpsc::channel();
    let delay = time::Duration::from_millis(500 - (opt.speed - 1) * 100);
    let link = Link {
        map: tx,
        message: tx_message,
    };

    let mut siv = Cursive::default();
    siv.set_fps(30);
    siv.add_global_callback('q', Cursive::quit);
    siv.load_toml(include_str!("theme.toml")).unwrap();
    siv.add_layer(
        LinearLayout::vertical()
            .child(TextView::new(format!("Map: {:?}", &opt.input)).fixed_height(1))
            .child(MessageView::new(rx_message).fixed_height(1))
            .child(MapView::new(rx, map.clone()).full_screen()),
    );

    thread::spawn(move || worker(map, link, delay));

    siv.run();

    Ok(())
}

#[derive(Debug)]
struct Link {
    map: mpsc::Sender<Map>,
    message: mpsc::Sender<String>,
}

fn worker(mut map: Map, link: Link, delay: time::Duration) {
    let result = map.run(|m, t| {
        thread::sleep(delay);
        link.map
            .send(m.clone())
            .map_err(|e| format_err!("Sending failed: {}", e))?;
        link.message
            .send(format!("Time: {}", t))
            .map_err(|e| format_err!("Sending failed: {}", e))?;
        Ok(())
    });

    link.map.send(map.clone()).unwrap();
    thread::sleep(delay);

    match result {
        Ok(outcome) => link.message.send(outcome.to_string()).unwrap(),
        Err(e) => link.message.send(format!("Error: {}", e)).unwrap(),
    };
}
