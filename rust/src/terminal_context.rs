use anyhow::Result;
use crossterm::terminal;

/// Terminal state and capabilities (read-only queries)
#[derive(Debug, Clone)]
pub struct TerminalContext {
    width: u16,
    height: u16,
    cursor_pos: (u16, u16),
    supports_colors: bool,
}

impl TerminalContext {
    /// Create new terminal context by querying current state
    pub fn new() -> Result<Self> {
        let (width, height) = terminal::size()?;

        Ok(Self {
            width,
            height,
            cursor_pos: (0, 0),
            supports_colors: true,
        })
    }

    /// Refresh terminal dimensions and cursor position
    pub fn refresh(&mut self) -> Result<()> {
        let (width, height) = terminal::size()?;
        self.width = width;
        self.height = height;
        Ok(())
    }

    /// Update cursor position
    pub fn set_cursor_pos(&mut self, col: u16, row: u16) {
        self.cursor_pos = (col, row);
    }

    /// Get terminal width
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Get terminal height
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Get current cursor position (col, row)
    pub fn cursor_pos(&self) -> (u16, u16) {
        self.cursor_pos
    }

    /// Check if terminal supports colors
    pub fn supports_colors(&self) -> bool {
        self.supports_colors
    }

    /// Check if given number of lines can fit in terminal
    pub fn can_fit(&self, lines: u16) -> bool {
        lines <= self.height
    }
}
