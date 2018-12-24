use std::convert::TryFrom;
use std::fmt::Debug;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use cursive::event::{Event, EventResult, Key};
use cursive::traits::*;
use cursive::vec::Vec2;
use cursive::Printer;

use geometry::{BoundingBox, Point};

pub trait Map<T>: Debug {
    fn display(&self, printer: &Printer, point: Point);

    fn bbox(&self) -> BoundingBox;

    fn update(&mut self, item: &T);
}

#[derive(Debug)]
pub struct Offset {
    pub left: isize,
    pub top: isize,
}

impl Offset {
    pub fn new() -> Self {
        Self { left: 0, top: 0 }
    }

    pub fn nudge(&mut self, left: isize, top: isize) {
        self.left = self.left.saturating_add(left);
        self.top = self.top.saturating_add(top);
    }

    pub fn event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Key(Key::Up) => self.nudge(0, -1),
            Event::Key(Key::Down) => self.nudge(0, 1),
            Event::Key(Key::Left) => self.nudge(-1, 0),
            Event::Key(Key::Right) => self.nudge(1, 0),
            _ => return EventResult::Ignored,
        }
        EventResult::Consumed(None)
    }
}

#[derive(Debug)]
pub struct OneReceiver<T> {
    rx: mpsc::Receiver<T>,
    delay: Option<Duration>,
    last_update: Option<Instant>,
    item: Option<T>,
}

fn since(i: Instant) -> Duration {
    Instant::now().duration_since(i)
}

impl<T> OneReceiver<T> {
    pub fn new(rx: mpsc::Receiver<T>) -> Self {
        Self {
            rx,
            delay: None,
            last_update: None,
            item: None,
        }
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = Some(delay);
        self
    }

    pub fn update(&mut self) -> bool {
        if let Some(delay) = self.delay {
            if self.last_update.map_or(false, |i| since(i) < delay) {
                return false;
            }
        }

        if let Ok(item) = self.rx.try_recv() {
            self.item = Some(item);
            self.last_update = Some(Instant::now());
            true
        } else {
            false
        }
    }

    pub fn get(&self) -> Option<&T> {
        self.item.as_ref()
    }
}

#[derive(Debug)]
pub struct MapView<T, Q>
where
    T: Map<Q>,
{
    map: T,
    offset: Offset,
    rx: OneReceiver<Q>,
}

impl<T, Q> MapView<T, Q>
where
    T: Map<Q>,
{
    pub fn new(rx: mpsc::Receiver<Q>, map: T) -> Self {
        Self {
            rx: OneReceiver::new(rx),
            offset: Offset::new(),
            map: map,
        }
    }

    fn update(&mut self) {
        // Read as much of the queue as possible.
        while self.rx.update() {
            if let Some(item) = self.rx.get() {
                self.map.update(item)
            }
        }
    }
}

impl<T, Q> View for MapView<T, Q>
where
    T: 'static + Map<Q>,
    Q: 'static,
{
    fn layout(&mut self, _: Vec2) {
        // Before drawing, we'll want to update the map
        self.update();
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        self.offset.event(event)
    }

    fn draw(&self, printer: &Printer) {
        let top = usize::try_from(self.offset.top).unwrap_or(0);
        let left = usize::try_from(self.offset.left).unwrap_or(0);

        let bbox = self.map.bbox().margin(1);

        let mx = printer.output_size.x;
        let my = printer.output_size.y;

        for (j, y) in bbox.vertical().skip(top).take(my).enumerate() {
            for (i, x) in bbox.horizontal().skip(left).take(mx).enumerate() {
                self.map.display(&printer.offset((i, j)), Point::new(x, y));
            }
        }
    }
}
