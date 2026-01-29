use anyhow::Result;
use crossterm::{
    cursor, execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{stdout, Stdout, Write};

use crate::node::Node;
use crate::terminal_context::TerminalContext;

/// Low-level terminal rendering operations
pub struct Renderer {
    stdout: Stdout,
    ctx: TerminalContext,
    /// Current rendering position (column)
    current_x: u16,
    /// Current rendering position (row)
    current_y: u16,
    /// Starting row for inline rendering (where we began)
    start_row: u16,
}

impl Renderer {
    /// Create a new renderer with the given terminal context
    pub fn new(ctx: TerminalContext) -> Self {
        Self {
            stdout: stdout(),
            ctx,
            current_x: 0,
            current_y: 0,
            start_row: 0,
        }
    }

    /// Set the starting row for rendering
    pub fn set_start_row(&mut self, row: u16) {
        self.start_row = row;
        self.current_x = 0;
        self.current_y = row;
    }

    /// Get current rendering position
    pub fn current_position(&self) -> (u16, u16) {
        (self.current_x, self.current_y)
    }

    /// Reset rendering position to start
    pub fn reset_position(&mut self) {
        self.current_x = 0;
        self.current_y = self.start_row;
    }

    /// Move to new line in rendering flow
    pub fn new_line(&mut self) {
        self.current_x = 0;
        self.current_y += 1;
    }

    /// Render a node at current position and update position
    pub fn render_node(&mut self, node: &mut dyn Node) -> Result<()> {
        let options = node.options();

        // Handle new_line option
        if options.new_line {
            self.new_line();
        }

        // Save where we're rendering this node
        let render_x = self.current_x;
        let render_y = self.current_y;

        // Get content lines
        let lines = node.content();

        // Render each line
        for (line_idx, line) in lines.iter().enumerate() {
            let line_y = render_y + line_idx as u16;

            // Don't render if outside terminal bounds
            if line_y >= self.ctx.height() {
                break;
            }

            // Move to position and render
            if let (Some(fg), Some(bg)) = (options.fg_color, options.bg_color) {
                self.draw_colored_at(render_x, line_y, line, fg, Some(bg))?;
            } else if let Some(fg) = options.fg_color {
                self.draw_colored_at(render_x, line_y, line, fg, None)?;
            } else {
                self.draw_at(render_x, line_y, line)?;
            }

            // Update current position based on this line
            if line_idx == lines.len() - 1 {
                // Last line - move cursor to end of it
                self.current_x = render_x + line.len() as u16;
                self.current_y = line_y;
            }
        }

        // Update node's position to where it was actually rendered
        node.set_position(render_x, render_y);

        // If multi-line node, we're now at the end of last line
        if lines.len() > 1 {
            self.current_y = render_y + (lines.len() - 1) as u16;
        }

        Ok(())
    }

    /// Get reference to terminal context
    pub fn context(&self) -> &TerminalContext {
        &self.ctx
    }

    /// Get mutable reference to terminal context
    pub fn context_mut(&mut self) -> &mut TerminalContext {
        &mut self.ctx
    }

    /// Refresh terminal context (size, etc.)
    pub fn refresh_context(&mut self) -> Result<()> {
        self.ctx.refresh()
    }

    /// Clear the entire screen
    pub fn clear_screen(&mut self) -> Result<()> {
        execute!(self.stdout, terminal::Clear(ClearType::All))?;
        Ok(())
    }

    /// Clear from cursor to end of screen
    pub fn clear_from_cursor(&mut self) -> Result<()> {
        execute!(self.stdout, terminal::Clear(ClearType::FromCursorDown))?;
        Ok(())
    }

    /// Clear current line
    pub fn clear_line(&mut self) -> Result<()> {
        execute!(self.stdout, terminal::Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    /// Move cursor to position (col, row)
    pub fn move_to(&mut self, col: u16, row: u16) -> Result<()> {
        execute!(self.stdout, cursor::MoveTo(col, row))?;
        Ok(())
    }

    /// Move cursor up by n lines
    pub fn move_up(&mut self, n: u16) -> Result<()> {
        if n > 0 {
            execute!(self.stdout, cursor::MoveUp(n))?;
        }
        Ok(())
    }

    /// Move cursor down by n lines
    pub fn move_down(&mut self, n: u16) -> Result<()> {
        if n > 0 {
            execute!(self.stdout, cursor::MoveDown(n))?;
        }
        Ok(())
    }

    /// Move cursor left by n columns
    pub fn move_left(&mut self, n: u16) -> Result<()> {
        if n > 0 {
            execute!(self.stdout, cursor::MoveLeft(n))?;
        }
        Ok(())
    }

    /// Move cursor right by n columns
    pub fn move_right(&mut self, n: u16) -> Result<()> {
        if n > 0 {
            execute!(self.stdout, cursor::MoveRight(n))?;
        }
        Ok(())
    }

    /// Hide cursor
    pub fn hide_cursor(&mut self) -> Result<()> {
        execute!(self.stdout, cursor::Hide)?;
        Ok(())
    }

    /// Show cursor
    pub fn show_cursor(&mut self) -> Result<()> {
        execute!(self.stdout, cursor::Show)?;
        Ok(())
    }

    /// Draw text at current cursor position
    pub fn draw(&mut self, text: &str) -> Result<()> {
        execute!(self.stdout, Print(text))?;
        Ok(())
    }

    /// Draw text at specific position
    pub fn draw_at(&mut self, col: u16, row: u16, text: &str) -> Result<()> {
        self.move_to(col, row)?;
        self.draw(text)?;
        Ok(())
    }

    /// Draw colored text at current position
    pub fn draw_colored(&mut self, text: &str, fg: Color, bg: Option<Color>) -> Result<()> {
        execute!(self.stdout, SetForegroundColor(fg))?;
        if let Some(bg_color) = bg {
            execute!(self.stdout, SetBackgroundColor(bg_color))?;
        }
        execute!(self.stdout, Print(text), ResetColor)?;
        Ok(())
    }

    /// Draw colored text at specific position
    pub fn draw_colored_at(
        &mut self,
        col: u16,
        row: u16,
        text: &str,
        fg: Color,
        bg: Option<Color>,
    ) -> Result<()> {
        self.move_to(col, row)?;
        self.draw_colored(text, fg, bg)?;
        Ok(())
    }

    /// Draw a filled character at position with colors
    pub fn draw_cell(&mut self, col: u16, row: u16, ch: char, fg: Color, bg: Color) -> Result<()> {
        self.move_to(col, row)?;
        execute!(
            self.stdout,
            SetForegroundColor(fg),
            SetBackgroundColor(bg),
            Print(ch),
            ResetColor
        )?;
        Ok(())
    }

    /// Flush the output buffer
    pub fn flush(&mut self) -> Result<()> {
        self.stdout.flush()?;
        Ok(())
    }

    /// Enter raw mode (required for reading individual key events)
    pub fn enter_raw_mode(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        Ok(())
    }

    /// Exit raw mode
    pub fn exit_raw_mode(&mut self) -> Result<()> {
        terminal::disable_raw_mode()?;
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        // Ensure we exit raw mode and show cursor on drop
        let _ = self.show_cursor();
        let _ = terminal::disable_raw_mode();
    }
}
