#![feature(try_from)]

extern crate carts;

#[macro_use]
extern crate serde_derive;

extern crate cursive;

extern crate docopt;

use std::convert::TryFrom;
use std::error::Error;
use std::fs;
use std::io::prelude::*;
use std::path::Path;
use std::process::exit;
use std::sync::mpsc;
use std::{thread, time};

use docopt::Docopt;

use geometry::Point;

use carts::Direction as CDirection;
use carts::{Layout, LayoutComplete, LayoutError};

use cursive::event::{Event, EventResult, Key};
use cursive::theme::ColorStyle;
use cursive::traits::*;
use cursive::vec::Vec2;
use cursive::view::Selector;
use cursive::views::LinearLayout;
use cursive::{Cursive, Printer};

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

fn build_ui(
    rx_layout: mpsc::Receiver<Layout>,
    rx_message: mpsc::Receiver<String>,
) -> Result<Cursive, Box<Error>> {
    let mut siv = Cursive::default();

    siv.set_fps(30);
    siv.load_toml(include_str!("theme.toml"))
        .map_err(|e| format!("{:?}", e))?;

    siv.add_global_callback('q', |s| s.quit());

    let mut boxes = LinearLayout::vertical();

    boxes.add_child(MessageView::new(rx_message).fixed_height(1));
    boxes.add_child(LayoutView::new(rx_layout).full_screen().with_id("layout"));

    siv.add_layer(boxes.full_screen());
    siv.focus(&Selector::Id("layout")).unwrap();
    Ok(siv)
}

fn run() -> Result<(), Box<Error>> {
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

struct Offset {
    left: isize,
    top: isize,
}

impl Offset {
    fn nudge(&mut self, left: isize, top: isize) {
        self.left = self.left.saturating_add(left);
        self.top = self.top.saturating_add(top);
    }
}

struct LayoutView {
    layout: Layout,
    offset: Offset,
    // Receiving end of the stream
    rx: mpsc::Receiver<Layout>,
}

impl LayoutView {
    fn new(rx: mpsc::Receiver<Layout>) -> Self {
        LayoutView {
            rx: rx,
            offset: Offset { left: 0, top: 0 },
            layout: Layout::new(),
        }
    }

    // Reads available data from the stream into the buffer
    fn update(&mut self) {
        // Add each available line to the end of the buffer.
        while let Ok(layout) = self.rx.try_recv() {
            self.layout = layout;
        }
    }
}

impl View for LayoutView {
    fn layout(&mut self, _: Vec2) {
        // Before drawing, we'll want to update the buffer
        self.update();
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        // Each line will be a debug-format of the event.

        match event {
            Event::Key(Key::Up) => self.offset.nudge(0, -1),
            Event::Key(Key::Down) => self.offset.nudge(0, 1),
            Event::Key(Key::Left) => self.offset.nudge(-1, 0),
            Event::Key(Key::Right) => self.offset.nudge(1, 0),
            _ => return EventResult::Ignored,
        }

        EventResult::Consumed(None)
    }

    fn draw(&self, printer: &Printer) {
        // Print the end of the buffer

        let top = i32::try_from(self.offset.top).unwrap_or(0);
        let left = i32::try_from(self.offset.left).unwrap_or(0);

        let bbox = self.layout.bbox();
        let carts = self.layout.cart_mapping();

        for (j, y) in (top..bbox.height()).enumerate() {
            for (i, x) in (left..bbox.width()).enumerate() {
                let point = Point::new(x, y);

                if let Some(direction) = carts.get(&point) {
                    printer.with_color(ColorStyle::secondary(), |p| {
                        p.print((i, j), &CDirection::from(*direction).to_string())
                    });
                } else if let Some(track) = self.layout.get_track(&point) {
                    printer.print((i, j), &track.to_string());
                }
            }
        }
    }
}

struct MessageView {
    message: String,
    rx: mpsc::Receiver<String>,
}

impl MessageView {
    fn new(rx: mpsc::Receiver<String>) -> Self {
        Self {
            message: String::new(),
            rx: rx,
        }
    }

    fn update(&mut self) {
        // Add each available line to the end of the buffer.
        while let Ok(message) = self.rx.try_recv() {
            self.message = message;
        }
    }
}

impl View for MessageView {
    fn layout(&mut self, _: Vec2) {
        // Before drawing, we'll want to update the buffer
        self.update();
    }

    fn draw(&self, printer: &Printer) {
        // Print the end of the buffer
        printer.print((0, 0), &self.message);
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
