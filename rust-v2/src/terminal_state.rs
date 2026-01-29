use crossterm::terminal;
use std::io::Result;

use crate::position::Position;

#[derive(Clone, Debug)]
pub struct TerminalState {
    width: u16,
    height: u16,
    cursor: Position,
}

impl TerminalState {
    pub fn new() -> Result<Self> {
        let (width, height) = terminal::size()?;
        Ok(Self {
            width,
            height,
            cursor: Position::new(0, 0),
        })
    }

    pub fn refresh(&mut self) -> Result<bool> {
        let (width, height) = terminal::size()?;
        Ok(self.update_size(width, height))
    }

    pub fn update_size(&mut self, width: u16, height: u16) -> bool {
        let changed = self.width != width || self.height != height;
        self.width = width;
        self.height = height;
        changed
    }

    pub fn set_cursor(&mut self, x: u16, y: u16) {
        self.cursor.set(x, y);
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn cursor(&self) -> Position {
        self.cursor
    }
}
