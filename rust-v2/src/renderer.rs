use crate::layout::Layout;
use crate::step::Step;
use crate::style::Style;
use crate::view_state::{ErrorDisplay, ViewState};
use crossterm::{cursor, queue, terminal};
use crossterm::style::Color;
use std::io::{self, Write};
use unicode_width::UnicodeWidthStr;

struct RenderLine {
    spans: Vec<crate::span::Span>,
    cursor_offset: Option<usize>,
}

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

    pub fn render(
        &mut self,
        step: &Step,
        view_state: &ViewState,
        out: &mut impl Write,
    ) -> io::Result<()> {
        let (width, _) = terminal::size()?;
        let render_lines = self.build_render_lines(step, view_state);
        let frame = Layout::new().compose_spans(
            render_lines.iter().map(|line| line.spans.clone()),
            width,
        );
        let lines = frame.lines();
        let start = self.ensure_start_row(out, lines.len())?;
        queue!(out, cursor::Hide)?;
        self.draw_lines(out, start, lines)?;
        self.clear_extra_lines(out, start, lines.len())?;
        self.num_lines = lines.len();
        out.flush()?;

        let cursor_pos = self.find_cursor_position(&render_lines);
        if let Some((col, line_idx)) = cursor_pos {
            let cursor_row = start + line_idx as u16;
            queue!(out, cursor::MoveTo(col as u16, cursor_row))?;
        }
        queue!(out, cursor::Show)?;
        out.flush()?;

        Ok(())
    }

    fn find_cursor_position(&self, render_lines: &[RenderLine]) -> Option<(usize, usize)> {
        let mut line_idx = 0;

        for line in render_lines {
            if let Some(offset) = line.cursor_offset {
                return Some((offset, line_idx));
            }

            let newlines = line.spans.iter().filter(|s| s.text() == "\n").count();
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

    fn ensure_start_row(&mut self, out: &mut impl Write, line_count: usize) -> io::Result<u16> {
        if let Some(start) = self.start_row {
            return Ok(start);
        }

        let (_, row) = cursor::position()?;
        queue!(out, cursor::MoveTo(0, row))?;
        for _ in 0..line_count {
            writeln!(out)?;
        }
        out.flush()?;

        let (_, end_row) = cursor::position()?;
        let start = end_row.saturating_sub(line_count as u16);
        self.start_row = Some(start);
        self.num_lines = line_count;
        Ok(start)
    }

    fn draw_lines(
        &self,
        out: &mut impl Write,
        start: u16,
        lines: &[crate::frame::Line],
    ) -> io::Result<()> {
        for (idx, line) in lines.iter().enumerate() {
            let line_row = start + idx as u16;
            queue!(out, cursor::MoveTo(0, line_row))?;
            queue!(out, terminal::Clear(terminal::ClearType::CurrentLine))?;
            write!(out, "{}", line)?;
        }
        Ok(())
    }

    fn clear_extra_lines(
        &self,
        out: &mut impl Write,
        start: u16,
        current_len: usize,
    ) -> io::Result<()> {
        if current_len >= self.num_lines {
            return Ok(());
        }

        for idx in current_len..self.num_lines {
            let line_row = start + idx as u16;
            queue!(out, cursor::MoveTo(0, line_row))?;
            queue!(out, terminal::Clear(terminal::ClearType::CurrentLine))?;
        }
        Ok(())
    }

    fn build_render_lines(&self, step: &Step, view_state: &ViewState) -> Vec<RenderLine> {
        let mut lines = Vec::new();

        let prompt_style = Style::new().with_attribute(crossterm::style::Attribute::Bold);
        let hint_style = Style::new().with_fg(Color::DarkGrey);

        let input_nodes: Vec<&crate::node::Node> = step
            .nodes
            .iter()
            .filter(|n| matches!(n, crate::node::Node::Input(_)))
            .collect();

        let inline_prompt_input = input_nodes.len() == 1 && step.nodes.len() == 1;

        if !step.prompt.is_empty() {
            if inline_prompt_input {
                if let Some(node) = input_nodes.first() {
                    let inline_error = match node.as_input() {
                        Some(input) => matches!(
                            view_state.error_display(input.id()),
                            ErrorDisplay::InlineMessage
                        ),
                        None => false,
                    };
                    let mut spans = vec![
                        crate::span::Span::new(step.prompt.clone()).with_style(prompt_style),
                        crate::span::Span::new(" "),
                    ];
                    spans.extend(node.render_field(inline_error));
                    let prompt_width = step.prompt.width();
                    let cursor_offset = node
                        .cursor_offset_in_field()
                        .map(|offset| offset + prompt_width + 1);
                    lines.push(RenderLine { spans, cursor_offset });
                }
            } else {
                lines.push(RenderLine {
                    spans: vec![
                        crate::span::Span::new(step.prompt.clone()).with_style(prompt_style),
                    ],
                    cursor_offset: None,
                });
            }
        }

        if !(inline_prompt_input && !step.prompt.is_empty()) {
            for node in &step.nodes {
                let inline_error = match node.as_input() {
                    Some(input) => matches!(
                        view_state.error_display(input.id()),
                        ErrorDisplay::InlineMessage
                    ),
                    None => false,
                };
                let spans = node.render(inline_error);
                let cursor_offset = node.cursor_offset();
                lines.push(RenderLine { spans, cursor_offset });
            }
        }

        if let Some(hint) = &step.hint {
            if !hint.is_empty() {
                lines.push(RenderLine {
                    spans: vec![crate::span::Span::new(hint.clone()).with_style(hint_style)],
                    cursor_offset: None,
                });
            }
        }

        lines
    }
}
