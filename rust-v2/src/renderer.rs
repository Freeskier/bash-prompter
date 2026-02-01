use crate::frame::Frame;
use crate::layout::Placed;
use crate::node::{Node, NodeKind};
use crate::render_context::RenderContext;
use crate::span::Span;
use crate::step::Step;
use crossterm::style::Color;
use crossterm::{cursor, queue, terminal};
use std::io::{Result, Write};
use unicode_width::UnicodeWidthStr;

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

    /// Render a step with positioned nodes
    pub fn render<W: Write>(
        &mut self,
        step: &Step,
        placed: &[Placed],
        ctx: &RenderContext,
        out: &mut W,
    ) -> Result<()> {
        let (frame, cursor_pos) = self.build_frame(step, placed, ctx);
        let lines = frame.lines();

        // First render - reserve space by forcing scroll, then draw
        if self.start_row.is_none() {
            let (_, row) = cursor::position()?;

            // Hide cursor during render
            queue!(out, cursor::Hide)?;

            // Move to column 0 to start fresh
            queue!(out, cursor::MoveTo(0, row))?;

            // FORCE scroll by printing newlines FIRST (reserves scrollback!)
            for _ in 0..lines.len() {
                writeln!(out)?;
            }
            out.flush()?;

            // Now we know our start_row - after printing newlines, cursor moved down
            // So start_row is current_row - num_lines
            let (_, end_row) = cursor::position()?;
            let start = end_row.saturating_sub(lines.len() as u16);
            self.start_row = Some(start);
            self.num_lines = lines.len();

            // Now draw content by going back and overwriting the reserved lines
            for (idx, line) in lines.iter().enumerate() {
                let line_row = start + idx as u16;
                queue!(out, cursor::MoveTo(0, line_row))?;
                write!(out, "{}", line)?;
            }
            out.flush()?;

            // Position cursor correctly
            if let Some((col, line_idx)) = cursor_pos {
                let cursor_row = start + line_idx as u16;
                queue!(out, cursor::MoveTo(col, cursor_row))?;
            }
            queue!(out, cursor::Show)?;
            out.flush()?;
        } else if let Some(start) = self.start_row {
            // Re-render: go back to start and redraw in place

            queue!(out, cursor::Hide)?;

            // Redraw each line by moving to it and clearing
            for (idx, line) in lines.iter().enumerate() {
                let row = start + idx as u16;
                queue!(out, cursor::MoveTo(0, row))?;
                queue!(out, terminal::Clear(terminal::ClearType::CurrentLine))?;
                write!(out, "{}", line)?;
            }

            // Clear any extra lines if we have fewer now
            for idx in lines.len()..self.num_lines {
                let row = start + idx as u16;
                queue!(out, cursor::MoveTo(0, row))?;
                queue!(out, terminal::Clear(terminal::ClearType::CurrentLine))?;
            }

            self.num_lines = lines.len();

            // Position cursor
            if let Some((col, line_idx)) = cursor_pos {
                let cursor_row = start + line_idx as u16;
                queue!(out, cursor::MoveTo(col, cursor_row))?;
            }
            queue!(out, cursor::Show)?;
            out.flush()?;
        }

        Ok(())
    }

    /// Move cursor to end of rendered content (for cleanup)
    pub fn move_to_end(&self, _frame_lines: usize, out: &mut impl Write) -> Result<()> {
        if let Some(start) = self.start_row {
            let end_row = start + self.num_lines as u16;
            queue!(out, cursor::MoveTo(0, end_row))?;
            out.flush()?;
        }
        Ok(())
    }

    /// Build Frame from positioned nodes
    fn build_frame(
        &self,
        step: &Step,
        placed: &[Placed],
        ctx: &RenderContext,
    ) -> (Frame, Option<(u16, usize)>) {
        let mut frame = Frame::new();
        let mut cursor_pos: Option<(u16, usize)> = None;

        // Group placed nodes by line (y coordinate)
        let max_y = placed.iter().map(|p| p.y).max().unwrap_or(0);

        for line_y in 0..=max_y {
            frame.ensure_line();

            // Get all nodes on this line, sorted by x
            let mut line_nodes: Vec<_> = placed.iter().filter(|p| p.y == line_y).collect();
            line_nodes.sort_by_key(|p| p.x);

            let mut current_x = 0;

            for placed_node in line_nodes {
                let node = &step[placed_node.index];

                // Add padding if needed (gap before this node)
                let gap = placed_node.x.saturating_sub(current_x);
                if gap > 0 {
                    frame.current_line_mut().push(Span::new(" ".repeat(gap)));
                }

                // Render the node
                let (spans, cursor_offset) = self.render_node(node, ctx);

                // Track cursor position if this node is focused
                if let Some(offset) = cursor_offset {
                    let cursor_col = (placed_node.x + offset) as u16;
                    cursor_pos = Some((cursor_col, line_y));
                }

                // Add spans to frame
                for span in spans {
                    current_x += span.width();
                    frame.current_line_mut().push(span);
                }
            }

            // Move to next line (except for last line)
            if line_y < max_y {
                frame.new_line();
            }
        }

        frame.trim_trailing_empty();
        (frame, cursor_pos)
    }

    /// Render a single node to spans
    fn render_node(&self, node: &Node, ctx: &RenderContext) -> (Vec<Span>, Option<usize>) {
        match &node.kind {
            NodeKind::Text(text) => {
                let style = node.opts.to_style();
                let span = Span::new(text.text.clone()).with_style(style);
                (vec![span], None)
            }

            NodeKind::TextInput(text_input) => {
                let is_focused = ctx.is_focused(&text_input.input.id);
                self.render_text_input(text_input, &node.opts, is_focused)
            }
        }
    }

    /// Render TextInput node
    fn render_text_input(
        &self,
        text_input: &crate::node::TextInputNode,
        opts: &crate::node::Options,
        focused: bool,
    ) -> (Vec<Span>, Option<usize>) {
        let input = &text_input.input;
        let mut spans = Vec::new();
        let mut cursor_offset = None;

        // Base style from options
        let base_style = opts.to_style();

        // Label
        spans.push(Span::new(format!("{}: ", input.label)).with_style(base_style.clone()));
        let label_width = input.label.width() + 2; // ": "

        // Opening bracket if focused
        if focused {
            spans.push(Span::new("["));
        }

        // Check if we should display error instead of value
        if let Some(error_msg) = &input.error {
            // Display error: "✗ error message" in red, bold
            let error_text = format!("✗ {}", error_msg);
            let error_width = error_text.width();
            let display_width = input.min_width.max(error_width);
            let mut error_str = error_text;

            // Pad to min_width
            if error_width < display_width {
                error_str.push_str(&" ".repeat(display_width - error_width));
            }

            let error_style = base_style
                .clone()
                .with_fg(Color::Red)
                .with_attribute(crossterm::style::Attribute::Bold);

            spans.push(Span::new(error_str).with_style(error_style));

            // No cursor when showing error
            cursor_offset = None;
        } else {
            // Display value normally
            let value_width = input.value.width();
            let display_width = input.min_width.max(value_width);
            let mut value_str = input.value.clone();

            // Pad to min_width
            if value_width < display_width {
                value_str.push_str(&" ".repeat(display_width - value_width));
            }

            // Style: red if invalid and focused
            let value_style = if focused && input.validate().is_err() {
                base_style.clone().with_fg(Color::Red)
            } else {
                base_style.clone()
            };

            spans.push(Span::new(value_str).with_style(value_style));

            // Cursor position: label + "[" + width of text up to cursor
            if focused {
                let text_before_cursor: String =
                    input.value.chars().take(input.cursor_pos).collect();
                cursor_offset = Some(label_width + 1 + text_before_cursor.width());
            }
        }

        // Closing bracket if focused
        if focused {
            spans.push(Span::new("]"));
        }

        (spans, cursor_offset)
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
