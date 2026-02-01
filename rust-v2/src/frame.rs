use crate::span::Span;
use crossterm::style::{ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Line {
    spans: Vec<Span>,
}

impl Line {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, span: Span) {
        if !span.text().is_empty() {
            self.spans.push(span);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }

    pub fn width(&self) -> usize {
        self.spans.iter().map(|s| s.width()).sum()
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for span in &self.spans {
            // Zastosuj style jeśli są
            if let Some(fg) = span.style().fg() {
                write!(f, "{}", SetForegroundColor(fg))?;
            }
            if let Some(bg) = span.style().bg() {
                write!(f, "{}", SetBackgroundColor(bg))?;
            }

            // Zastosuj atrybuty (bold, italic, etc.)
            for attr in span.style().attributes() {
                write!(f, "{}", SetAttribute(*attr))?;
            }

            // Tekst
            write!(f, "{}", span.text())?;

            // Reset stylu
            if span.style().fg().is_some()
                || span.style().bg().is_some()
                || !span.style().attributes().is_empty()
            {
                write!(f, "{}", ResetColor)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Frame {
    lines: Vec<Line>,
}

impl Frame {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    pub fn lines_mut(&mut self) -> &mut Vec<Line> {
        &mut self.lines
    }

    pub fn ensure_line(&mut self) {
        if self.lines.is_empty() {
            self.lines.push(Line::new());
        }
    }

    pub fn current_line_mut(&mut self) -> &mut Line {
        self.ensure_line();
        self.lines.last_mut().unwrap()
    }

    pub fn new_line(&mut self) {
        self.lines.push(Line::new());
    }

    pub fn trim_trailing_empty(&mut self) {
        while self.lines.last().map(|l| l.is_empty()).unwrap_or(false) {
            self.lines.pop();
        }
        if self.lines.is_empty() {
            self.lines.push(Line::new());
        }
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, line) in self.lines.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", line)?;
        }
        Ok(())
    }
}
