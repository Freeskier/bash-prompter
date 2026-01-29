use crate::span::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    spans: Vec<Span>,
}

impl Line {
    pub fn new() -> Self {
        Self { spans: Vec::new() }
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

    pub fn to_string(&self) -> String {
        let mut out = String::new();
        for span in &self.spans {
            out.push_str(span.text());
        }
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    lines: Vec<Line>,
}

impl Frame {
    pub fn new() -> Self {
        Self { lines: Vec::new() }
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

    pub fn to_string(&self) -> String {
        self.lines
            .iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
