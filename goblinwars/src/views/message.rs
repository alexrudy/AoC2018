use std::sync::mpsc;

use cursive::traits::*;
use cursive::vec::Vec2;
use cursive::Printer;

#[derive(Debug)]
pub struct MessageView {
    message: String,
    rx: mpsc::Receiver<String>,
}

impl MessageView {
    pub fn create() -> (mpsc::Sender<String>, Self) {
        let (tx, rx) = mpsc::channel();
        let view = Self::new(rx);
        (tx, view)
    }

    pub fn new(rx: mpsc::Receiver<String>) -> Self {
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
