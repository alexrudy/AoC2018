#![feature(try_from)]

use serde_derive::Deserialize;

use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::path::Path;
use std::process::exit;
use std::sync::mpsc;
use std::{thread, time};

use docopt::Docopt;
use failure::{format_err, Error};

use geometry::{BoundingBox, Direction, Point};

use carts::Direction as CDirection;
use carts::{Layout, LayoutComplete, LayoutError};

use cursive::theme::ColorStyle;
use cursive::traits::*;
use cursive::view::Selector;
use cursive::views::LinearLayout;
use cursive::{Cursive, Printer};
use cursive_aoc_views::{Map, MapView, MessageView};

const USAGE: &str = "
Advent of Code 2018 - Elf Cart Visualizer.

Usage:
    playcart [--speed <s>] [--last|--collision] <layoutfile>
";

#[derive(Deserialize)]
struct Args {
    flag_speed: Option<u64>,
    flag_last: bool,
    flag_collision: bool,
    arg_layoutfile: String,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        exit(1);
    }
}

fn load_layout<P>(path: P) -> Result<Layout, Error>
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

fn build_ui(
    rx_layout: mpsc::Receiver<Layout>,
    rx_message: mpsc::Receiver<String>,
) -> Result<Cursive, Error> {
    let mut siv = Cursive::default();

    siv.set_fps(30);
    siv.load_toml(include_str!("theme.toml"))
        .map_err(|e| format_err!("{:?}", e))?;

    siv.add_global_callback('q', |s| s.quit());

    let mut boxes = LinearLayout::vertical();

    // let wait = time::Duration::from_millis(50);

    boxes.add_child(MessageView::new(rx_message).fixed_height(1));
    boxes.add_child(
        MapView::new(rx_layout, ViewableLayout::new(Layout::new()))
            .full_screen()
            .with_id("layout"),
    );

    siv.add_layer(boxes.full_screen());
    siv.focus(&Selector::Id("layout")).unwrap();
    Ok(siv)
}

fn run() -> Result<(), Error> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let layout = load_layout(args.arg_layoutfile)?;

    let (tx, rx) = mpsc::channel();
    let (tx_message, rx_message) = mpsc::channel();

    let mut siv = build_ui(rx, rx_message)?;
    let pause = time::Duration::from_millis(100 * (5 - args.flag_speed.unwrap_or(3)));

    let until = if args.flag_collision {
        LayoutComplete::Collision
    } else if args.flag_last {
        LayoutComplete::LastCart
    } else {
        LayoutComplete::Never
    };

    thread::spawn(move || {
        run_layout(layout, until, pause, &tx, &tx_message);
    });

    siv.run();

    Ok(())
}

#[derive(Debug)]
struct ViewableLayout {
    layout: Layout,
    carts: HashMap<Point, Direction>,
}

impl ViewableLayout {
    fn new(layout: Layout) -> Self {
        let carts = layout.cart_mapping();
        Self { layout, carts }
    }
}

impl Map<Layout> for ViewableLayout {
    fn display(&self, printer: &Printer, point: Point) {
        if let Some(direction) = self.carts.get(&point) {
            printer.with_color(ColorStyle::secondary(), |p| {
                p.print((0, 0), &CDirection::from(*direction).to_string())
            });
        } else if let Some(track) = self.layout.get_track(&point) {
            printer.print((0, 0), &track.to_string());
        }
    }

    fn bbox(&self) -> BoundingBox {
        self.layout.bbox()
    }

    fn update(&mut self, item: &Layout) {
        self.layout = item.clone();
        self.carts = self.layout.cart_mapping();
    }
}

fn run_layout(
    layout: Layout,
    until: LayoutComplete,
    pause: time::Duration,
    tx_layout: &mpsc::Sender<Layout>,
    tx_message: &mpsc::Sender<String>,
) {
    let mut layout = layout;
    let mut counter = 0..;
    let r = layout.run(
        |l| {
            if tx_layout.send(l.clone()).is_err() {
                return;
            }
            if tx_message
                .send(format!("Time: {}", counter.next().unwrap()))
                .is_err()
            {
                return;
            }
            thread::sleep(pause);
        },
        until,
    );

    if tx_layout.send(layout.clone()).is_err() {
        return;
    }

    match (r, until) {
        (Ok(()), _) => {}
        (Err(LayoutError::Collision(p)), LayoutComplete::Collision) => {
            tx_message
                .send(format!("Collision at {}", p))
                .expect("Error sending on stream:");
        }
        (Err(LayoutError::OneCart(p)), LayoutComplete::LastCart) => {
            tx_message
                .send(format!("Last cart at {}", p))
                .expect("Error sending on stream:");
        }
        (Err(e), _) => {
            tx_message
                .send(format!("Error: {}", e))
                .expect("Error sending on stream:");
        }
    };
}
