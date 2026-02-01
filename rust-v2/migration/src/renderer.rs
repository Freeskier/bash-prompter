use crate::frame::Frame;
use crate::step::Step;
use crossterm::{cursor, queue, terminal};
use std::io::{self, Write};

pub struct Renderer {
    start_row: Option<u16>,
    num_lines: usize,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            start_row: None,
            num_lines: 0,
        }
    }

    pub fn render(&mut self, step: &Step, out: &mut impl Write) -> io::Result<()> {
        let mut frame = Frame::new();

        for node in step {
            let spans = node.render();
            for span in spans {
                if span.text() == "\n" {
                    frame.new_line();
                } else {
                    frame.current_line_mut().push(span);
                }
            }
            frame.new_line();
        }

        frame.trim_trailing_empty();
        let lines = frame.lines();

        if self.start_row.is_none() {
            let (_, row) = cursor::position()?;
            queue!(out, cursor::Hide)?;
            queue!(out, cursor::MoveTo(0, row))?;

            for _ in 0..lines.len() {
                writeln!(out)?;
            }
            out.flush()?;

            let (_, end_row) = cursor::position()?;
            let start = end_row.saturating_sub(lines.len() as u16);
            self.start_row = Some(start);
            self.num_lines = lines.len();

            for (idx, line) in lines.iter().enumerate() {
                let line_row = start + idx as u16;
                queue!(out, cursor::MoveTo(0, line_row))?;
                queue!(out, terminal::Clear(terminal::ClearType::CurrentLine))?;
                write!(out, "{}", line)?;
            }
            out.flush()?;

            let cursor_pos = self.find_cursor_position(step);
            if let Some((col, line_idx)) = cursor_pos {
                let cursor_row = start + line_idx as u16;
                queue!(out, cursor::MoveTo(col as u16, cursor_row))?;
            }
            queue!(out, cursor::Show)?;
            out.flush()?;
        } else if let Some(start) = self.start_row {
            queue!(out, cursor::Hide)?;

            for (idx, line) in lines.iter().enumerate() {
                let line_row = start + idx as u16;
                queue!(out, cursor::MoveTo(0, line_row))?;
                queue!(out, terminal::Clear(terminal::ClearType::CurrentLine))?;
                write!(out, "{}", line)?;
            }

            if lines.len() < self.num_lines {
                for idx in lines.len()..self.num_lines {
                    let line_row = start + idx as u16;
                    queue!(out, cursor::MoveTo(0, line_row))?;
                    queue!(out, terminal::Clear(terminal::ClearType::CurrentLine))?;
                }
            }

            self.num_lines = lines.len();
            out.flush()?;

            let cursor_pos = self.find_cursor_position(step);
            if let Some((col, line_idx)) = cursor_pos {
                let cursor_row = start + line_idx as u16;
                queue!(out, cursor::MoveTo(col as u16, cursor_row))?;
            }
            queue!(out, cursor::Show)?;
            out.flush()?;
        }

        Ok(())
    }

    fn find_cursor_position(&self, step: &Step) -> Option<(usize, usize)> {
        let mut line_idx = 0;

        for node in step {
            if let Some(offset) = node.cursor_offset() {
                return Some((offset, line_idx));
            }

            let spans = node.render();
            let newlines = spans.iter().filter(|s| s.text() == "\n").count();
            line_idx += 1 + newlines;
        }

        None
    }

    pub fn move_to_end(&self, out: &mut impl Write) -> io::Result<()> {
        if let Some(start) = self.start_row {
            let end_row = start + self.num_lines as u16;
            queue!(out, cursor::MoveTo(0, end_row))?;
            out.flush()?;
        }
        Ok(())
    }
}
