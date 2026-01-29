use crate::frame::Frame;
use crossterm::{cursor, queue, terminal};
use std::io::{Result, Write};

pub struct Renderer {
    anchor_row: Option<u16>,
    last_frame_lines: usize,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            anchor_row: None,
            last_frame_lines: 0,
        }
    }

    pub fn render_to<W: Write>(&mut self, frame: &Frame, out: &mut W) -> Result<()> {
        if self.anchor_row.is_none() {
            let (_, row) = cursor::position()?;
            self.anchor_row = Some(row);
        }

        let anchor = self.anchor_row.unwrap();

        queue!(out, cursor::MoveTo(0, anchor))?;
        queue!(out, terminal::Clear(terminal::ClearType::FromCursorDown))?;

        for (idx, line) in frame.lines().iter().enumerate() {
            let row = anchor + idx as u16;
            queue!(out, cursor::MoveTo(0, row))?;
            write!(out, "{}", line.to_string())?;
        }

        let end_row = anchor + frame.lines().len() as u16;
        queue!(out, cursor::MoveTo(0, end_row))?;
        out.flush()?;

        self.last_frame_lines = frame.lines().len();
        Ok(())
    }
}
