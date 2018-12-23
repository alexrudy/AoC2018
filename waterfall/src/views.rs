use std::cmp;
use std::convert::TryFrom;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::{WaterLevel, WellSystem};

use cursive::event::{Event, EventResult};
use cursive::theme::{BaseColor, Color, ColorStyle, ColorType};
use cursive::traits::*;
use cursive::vec::Vec2;
use cursive::views::LinearLayout;
use cursive::Printer;

use cursive_aoc_views::{Map, MessageView, Offset, OneReceiver};
use geometry::{BoundingBox, Point};

impl Map<WaterLevel> for WellSystem {
    fn display(&self, printer: &Printer, point: Point) {
        if point == self.ground.spring {
            printer.print((0, 0), "+");
        }
        if let Some(f) = self.flow(point) {
            let mut color_style = ColorStyle::background();
            color_style.front = ColorType::Color(Color::Light(BaseColor::Blue));

            printer.with_color(color_style, |p| p.print((0, 0), &format!("{}", f)));
        } else if self.ground.contains(point) {
            printer.print((0, 0), "#");
        }
    }

    fn bbox(&self) -> BoundingBox {
        self.bbox()
    }

    fn update(&mut self, item: &WaterLevel) {
        self.insert(item);
    }
}

struct Comms {
    map: mpsc::Sender<WaterLevel>,
    message: mpsc::Sender<String>,
}

impl Comms {
    fn send_map(&self, item: WaterLevel) {
        match self.map.send(item) {
            Ok(_) => {}
            Err(mpsc::SendError(_)) => self.message.send("Error: Hangup".to_string()).unwrap(),
        }
    }
}

pub fn layout(system: WellSystem, _: Duration) -> LinearLayout {
    let (tx_layout, rx_layout) = mpsc::channel();
    let (tx_message, rx_message) = mpsc::channel();

    let boxes = LinearLayout::vertical()
        .child(MessageView::new(rx_message).fixed_height(1))
        .child(
            WaterView::new(rx_layout, system.clone())
                .full_screen()
                .with_id("layout"),
        );

    let comms = Comms {
        map: tx_layout,
        message: tx_message,
    };

    thread::spawn(move || process(system, &comms));

    boxes
}

fn process(mut system: WellSystem, comms: &Comms) {
    for (t, s) in system.fill().enumerate() {
        thread::sleep(Duration::from_millis(10));
        comms.send_map(s);
        comms.message.send(format!("Time: {:5}", t)).unwrap();
    }
}

#[derive(Debug)]
pub struct WaterView {
    map: WellSystem,
    offset: Offset,
    rx: OneReceiver<WaterLevel>,
    paused: bool,
}

impl WaterView {
    pub fn new(rx: mpsc::Receiver<WaterLevel>, map: WellSystem) -> Self {
        Self {
            rx: OneReceiver::new(rx),
            offset: Offset::new(),
            map: map,
            paused: false,
        }
    }

    fn update(&mut self) {
        // Read as much of the queue as possible.
        while self.rx.update() {
            if let Some(item) = self.rx.get() {
                if !self.paused {
                    self.map.update(item)
                }
            }
        }
    }
}

impl View for WaterView {
    fn layout(&mut self, _: Vec2) {
        // Before drawing, we'll want to update the map
        self.update();
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        if let Event::Char(' ') = event {
            self.paused = !self.paused;
            return EventResult::Consumed(None);
        }

        self.offset.event(event)
    }

    fn draw(&self, printer: &Printer) {
        let bbox = self.map.bbox().margin(1);

        let mx = printer.output_size.x;
        let my = printer.output_size.y;

        let water = self
            .map
            .water
            .level_bbox()
            .unwrap_or_else(BoundingBox::zero);

        let left = usize::try_from(cmp::max(
            bbox.left() + i32::try_from(self.offset.left).unwrap_or(0),
            water.left() - (i32::try_from(mx).unwrap_or(0) / 2),
        ))
        .unwrap_or(0);

        let top = usize::try_from(cmp::max(
            bbox.top() + i32::try_from(self.offset.top).unwrap_or(0),
            water.top() - (i32::try_from(my).unwrap_or(0) / 2),
        ))
        .unwrap_or(0);

        // eprintln!("{},{}, {:?}", left, top, water);

        for (j, y) in (top..).take(my).enumerate() {
            for (i, x) in (left..).take(mx).enumerate() {
                self.map
                    .display(&printer.offset((i, j)), Point::new(x as i32, y as i32));
            }
        }

        if let Some(item) = self.rx.get() {
            for f in &item.flows {
                let mut color_style = ColorStyle::background();
                color_style.front = ColorType::Color(Color::Light(BaseColor::Blue));

                let x = f.x - (left as i32);
                let y = f.y - (top as i32);

                if x >= 0 && y >= 0 {
                    printer.with_color(color_style, |p| p.print((x as usize, y as usize), "#"));
                }
            }
        }
    }
}
