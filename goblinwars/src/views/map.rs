use std::convert::TryFrom;
use std::sync::mpsc;

use cursive::event::{Event, EventResult, Key};
use cursive::theme::ColorStyle;
use cursive::traits::*;
use cursive::vec::Vec2;
use cursive::Printer;

use crate::map::{Map, MapElement};
use crate::sprite::Species;
use geometry::Point;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct MapView {
    map: Map,
    offset: Offset,
    // Receiving end of the stream
    rx: mpsc::Receiver<Map>,
}

impl MapView {
    pub fn new(rx: mpsc::Receiver<Map>, map: Map) -> Self {
        Self {
            rx: rx,
            offset: Offset { left: 0, top: 0 },
            map: map,
        }
    }

    // Reads available data from the stream into the buffer
    fn update(&mut self) {
        while let Ok(map) = self.rx.try_recv() {
            self.map = map;
        }
    }
}

impl View for MapView {
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

        let top = usize::try_from(self.offset.top).unwrap_or(0);
        let left = usize::try_from(self.offset.left).unwrap_or(0);

        let bbox = self.map.bbox().margin(1);
        let mut sprites = Vec::new();

        for (j, y) in bbox.vertical().skip(top).enumerate() {
            sprites.clear();
            for (i, x) in bbox.horizontal().skip(left).enumerate() {
                let point = Point::new(x, y);

                match self.map.element(point) {
                    MapElement::Sprite(sprite) => {
                        let style = match sprite {
                            Species::Elf => ColorStyle::secondary(),
                            Species::Goblin => ColorStyle::tertiary(),
                        };
                        printer.with_color(style, |p| p.print((i, j), &sprite.to_string()));
                        sprites.push(self.map.sprites.get(point).unwrap().info());
                    }
                    MapElement::Tile(tile) => {
                        printer.print((i, j), &tile.to_string());
                    }
                }
            }

            for (i, info) in sprites.iter().enumerate() {
                if i > 0 {
                    let p = 1 + (i * 8) + usize::try_from(bbox.right()).unwrap_or(0) - left;
                    printer.print((p, j), ",");
                }

                let p = 3 + (i * 8) + usize::try_from(bbox.right()).unwrap_or(0) - left;
                printer.print((p, j), &info.to_string());
            }
        }
    }
}
