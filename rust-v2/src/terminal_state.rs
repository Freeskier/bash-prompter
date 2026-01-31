use crossterm::terminal;
use std::io::Result;

#[derive(Clone, Debug)]
pub struct TerminalState {
    width: u16,
    height: u16,
}

impl TerminalState {
    pub fn new() -> Result<Self> {
        let (width, height) = terminal::size()?;
        Ok(Self { width, height })
    }

    pub fn refresh(&mut self) -> Result<bool> {
        let (width, height) = terminal::size()?;
        Ok(self.update_size(width, height))
    }

    fn update_size(&mut self, width: u16, height: u16) -> bool {
        let changed = self.width != width || self.height != height;
        self.width = width;
        self.height = height;
        changed
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }
}
