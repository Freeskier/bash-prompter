use crate::frame::Frame;
use crossterm::{cursor, queue, terminal};
use std::io::{Result, Write};

pub struct Renderer {
    anchor_row: Option<u16>,
}

impl Renderer {
    pub fn new() -> Self {
        Self { anchor_row: None }
    }

    pub fn render<W: Write>(&mut self, frame: &Frame, out: &mut W) -> Result<()> {
        let anchor = self.get_or_init_anchor()?;
        self.clear_and_render(frame, anchor, out)
    }

    pub fn render_at<W: Write>(&mut self, frame: &Frame, anchor: u16, out: &mut W) -> Result<()> {
        self.anchor_row = Some(anchor);
        self.clear_and_render(frame, anchor, out)
    }

    fn get_or_init_anchor(&mut self) -> Result<u16> {
        if let Some(anchor) = self.anchor_row {
            Ok(anchor)
        } else {
            let (_, row) = cursor::position()?;
            self.anchor_row = Some(row);
            Ok(row)
        }
    }

    fn clear_and_render<W: Write>(&self, frame: &Frame, anchor: u16, out: &mut W) -> Result<()> {
        queue!(out, cursor::MoveTo(0, anchor))?;
        queue!(out, terminal::Clear(terminal::ClearType::FromCursorDown))?;

        for (idx, line) in frame.lines().iter().enumerate() {
            let row = anchor + idx as u16;
            queue!(out, cursor::MoveTo(0, row))?;
            write!(out, "{}", line)?;
        }

        queue!(out, cursor::MoveTo(0, anchor))?;
        out.flush()
    }

    pub fn anchor_row(&self) -> Option<u16> {
        self.anchor_row
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
